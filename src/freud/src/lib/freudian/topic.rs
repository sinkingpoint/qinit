use super::subscriber::Subscription;

#[derive(PartialEq)]
pub enum TopicState {
    Steady,
    MarkedForDeletion,
    MarkedForRecreation
}

pub struct Topic {
    pub name: String,
    pub state: TopicState,
    pub subscribers: Vec<Subscription>
}

impl Topic {
    pub fn new(name: String) -> Topic {
        return Topic {
            name: name,
            state: TopicState::Steady,
            subscribers: Vec::new()
        };
    }

    pub fn add_subscriber(&mut self) -> &Subscription {
        self.subscribers.push(Subscription::new().unwrap());
        return self.subscribers.last().unwrap();
    }
}
