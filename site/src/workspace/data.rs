use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub project: u64,
    pub title: String,
    pub note: String,
    pub done: bool,
    pub priority: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub project: u64,
    pub body: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Data {
    pub version: u32,
    #[serde(default)]
    pub intro_dismissed: bool,
    pub next_id: u64,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub messages: Vec<Message>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            version: 1,
            intro_dismissed: false,
            next_id: 100,
            projects: vec![
                Project { id: 1, name: "Website launch".into() },
                Project { id: 2, name: "Personal".into() },
                Project { id: 8, name: "Reading room".into() },
            ],
            tasks: [
                (3, 1, "Make a great first impression", "Bring the home page together: a clear introduction, thoughtful type, and one strong call to action. The first screen should feel as considered as the product itself.", false, true),
                (4, 1, "Find the right words", "Write a short introduction that sounds like us. Say what we make and why it matters.", false, true),
                (5, 1, "Polish the little interactions", "Review hover, focus, and pressed states. Keep keyboard actions instant and motion purposeful.", false, false),
                (6, 1, "Give the mobile layout some love", "Check the navigation, reading width, and spacing on a smaller screen.", false, false),
                (7, 1, "Choose the project photography", "Pick a small set of images that tell the story. Keep the palette and cropping consistent.", false, false),
                (9, 1, "Check every link", "Walk through the site as a new visitor. Check links, downloads, and the contact flow.", false, false),
                (10, 1, "Collect a few good references", "Save ideas worth learning from, with a note about what works.", true, false),
                (11, 1, "Decide what belongs on the page", "Keep the structure clear and the content useful.", true, false),
                (12, 2, "Plan a slow weekend", "A walk, a good meal, and a little time with no plans.", false, false),
                (13, 2, "Make something just for fun", "Leave room for a small project with no deadline.", false, false),
                (14, 8, "Pick the next book", "Choose one book and make time for the first chapter.", false, false),
            ].into_iter().map(|(id, project, title, note, done, priority)| Task {
                id, project, title: title.into(), note: note.into(), done, priority,
            }).collect(),
            messages: Vec::new(),
        }
    }
}

impl Data {
    pub fn validate(&self) -> bool {
        let projects: HashSet<_> = self.projects.iter().map(|p| p.id).collect();
        let mut ids = HashSet::new();
        self.version == 1
            && !self.projects.is_empty()
            && self.next_id < u64::MAX
            && self
                .projects
                .iter()
                .all(|p| !p.name.trim().is_empty() && p.name.len() <= 120)
            && self.tasks.iter().all(|t| {
                projects.contains(&t.project)
                    && !t.title.trim().is_empty()
                    && t.title.len() <= 500
                    && t.note.len() <= 10_000
            })
            && self.messages.iter().all(|m| {
                projects.contains(&m.project) && !m.body.trim().is_empty() && m.body.len() <= 10_000
            })
            && self
                .projects
                .iter()
                .map(|p| p.id)
                .chain(self.tasks.iter().map(|t| t.id))
                .chain(self.messages.iter().map(|m| m.id))
                .all(|id| id < self.next_id && ids.insert(id))
    }

    pub fn add_project(&mut self, name: &str) -> Option<u64> {
        let name = name.trim();
        if name.is_empty()
            || name.len() > 120
            || self
                .projects
                .iter()
                .any(|p| p.name.eq_ignore_ascii_case(name))
        {
            return None;
        }
        let id = self.allocate()?;
        self.projects.push(Project {
            id,
            name: name.into(),
        });
        Some(id)
    }

    pub fn add_task(&mut self, project: u64, title: &str) -> Option<u64> {
        let title = title.trim();
        if title.is_empty() || title.len() > 500 || !self.projects.iter().any(|p| p.id == project) {
            return None;
        }
        let id = self.allocate()?;
        self.tasks.push(Task {
            id,
            project,
            title: title.into(),
            note: String::new(),
            done: false,
            priority: false,
        });
        Some(id)
    }

    pub fn send(&mut self, project: u64, body: &str) -> Option<u64> {
        let body = body.trim();
        if body.is_empty() || body.len() > 10_000 || !self.projects.iter().any(|p| p.id == project)
        {
            return None;
        }
        let id = self.allocate()?;
        self.messages.push(Message {
            id,
            project,
            body: body.into(),
        });
        Some(id)
    }

    fn allocate(&mut self) -> Option<u64> {
        let id = self.next_id;
        self.next_id = id.checked_add(1)?;
        Some(id)
    }
}

pub struct Store {
    pub path: PathBuf,
    previous: Option<Vec<u8>>,
    blocked: bool,
}

impl Store {
    pub fn open(path: PathBuf) -> (Self, Data, Option<String>) {
        let mut store = Self {
            path,
            previous: None,
            blocked: false,
        };
        match fs::read(&store.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => (store, Data::default(), None),
            Ok(bytes) => {
                let data = (bytes.len() <= 5_000_000)
                    .then(|| serde_json::from_slice::<Data>(&bytes).ok())
                    .flatten()
                    .filter(Data::validate);
                store.previous = Some(bytes);
                if let Some(data) = data {
                    (store, data, None)
                } else {
                    store.blocked = true;
                    (store, Data::default(), Some("Saved workspace could not be read. Your file is untouched; changes will not be saved.".into()))
                }
            }
            Err(error) => {
                store.blocked = true;
                (
                    store,
                    Data::default(),
                    Some(format!(
                        "Cannot read workspace: {error}. Changes will not be saved."
                    )),
                )
            }
        }
    }

    pub fn save(&mut self, data: &Data) -> io::Result<()> {
        if self.blocked || !data.validate() {
            return Err(io::Error::other(
                "Saved file needs attention; original data has been preserved",
            ));
        }
        let current = match fs::read(&self.path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        if current != self.previous {
            return Err(io::Error::other(
                "Workspace changed in another window. Reopen the app before saving",
            ));
        }
        let bytes = serde_json::to_vec_pretty(data)?;
        if bytes.len() > 5_000_000 {
            return Err(io::Error::other(
                "This example supports up to 5 MB of workspace data",
            ));
        }
        let parent = self
            .path
            .parent()
            .ok_or_else(|| io::Error::other("Missing data directory"))?;
        fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(&bytes)?;
        file.as_file().sync_all()?;
        file.persist(&self.path).map_err(|error| error.error)?;
        self.previous = Some(bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_workspace_round_trip_preserves_tasks_chat_and_rejects_bad_writes() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("workspace.json");
        let (mut store, mut data, error) = Store::open(path.clone());
        assert!(error.is_none());
        assert!(!data.intro_dismissed);
        let mut legacy = serde_json::to_value(&data).unwrap();
        legacy.as_object_mut().unwrap().remove("intro_dismissed");
        assert!(
            !serde_json::from_value::<Data>(legacy)
                .unwrap()
                .intro_dismissed
        );
        data.intro_dismissed = true;
        let project = data.add_project("Launch").unwrap();
        let task = data.add_task(project, "Publish the page").unwrap();
        data.tasks.iter_mut().find(|t| t.id == task).unwrap().done = true;
        data.send(project, "Remember to check the links").unwrap();
        assert!(data.add_task(999, "Bad project").is_none());
        assert!(data.send(project, " ").is_none());
        store.save(&data).unwrap();
        let (_, reopened, error) = Store::open(path.clone());
        assert!(error.is_none());
        assert!(reopened.intro_dismissed);
        assert!(reopened.tasks.iter().any(|t| t.id == task && t.done));
        assert_eq!(
            reopened.messages.last().unwrap().body,
            "Remember to check the links"
        );
        fs::write(&path, b"broken data").unwrap();
        assert!(store.save(&data).is_err());
        let (mut broken, _, error) = Store::open(path.clone());
        assert!(error.is_some() && broken.save(&data).is_err());
        assert_eq!(fs::read(path).unwrap(), b"broken data");
    }
}
