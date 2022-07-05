use std::collections::VecDeque;

pub(crate) struct Queue<T>
{
    queue: VecDeque<T>,
}

impl Queue<String> {
    pub(crate) fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub(crate) fn add_actions(&mut self, actions: Vec<String>) {
        self.queue.extend(actions);
    }

    pub(crate) fn get_next_action(&mut self) -> Option<String> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue() {
        let queue = Queue::new();
        assert_eq!(queue.queue.len(), 0);
    }

    #[test]
    fn test_add_actions_to_queue() {
        let mut queue = Queue::new();
        let actions = vec!["action1".to_string(), "action2".to_string()];

        queue.add_actions(actions);

        assert_eq!(queue.queue.len(), 2);
    }

    #[test]
    fn test_get_next_action_with_empty_queue() {
        let mut queue = Queue::new();
        assert_eq!(queue.get_next_action(), None);

        let next_action = queue.get_next_action();

        assert!(next_action.is_none());
    }

    #[test]
    fn test_get_next_action_with_one_element() {
        let mut queue = Queue::new();
        queue.add_actions(vec!["action1".to_string()]);

        let next_action = queue.get_next_action();

        assert!(next_action.is_some());
        assert_eq!(next_action.unwrap(), "action1".to_string());
    }
}
