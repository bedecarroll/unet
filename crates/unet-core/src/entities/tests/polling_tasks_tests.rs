//! Tests for `polling_tasks` entity

#[cfg(test)]
mod tests {
    use super::super::super::polling_tasks::*;
    use sea_orm::entity::prelude::*;
    use serde_json;

    #[test]
    fn test_polling_tasks_model_creation() {
        let task = Model {
            id: "task-001".to_string(),
            node_id: "node-001".to_string(),
            target: "192.168.1.1".to_string(),
            oids: r#"["1.3.6.1.2.1.1.3.0", "1.3.6.1.2.1.1.1.0"]"#.to_string(),
            interval_seconds: 300,
            session_config: r#"{"community": "public", "version": "v2c"}"#.to_string(),
            priority: 100,
            enabled: true,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            last_success: Some("2023-01-01T12:00:00Z".to_string()),
            last_error: None,
            consecutive_failures: 0,
        };

        assert_eq!(task.id, "task-001");
        assert_eq!(task.node_id, "node-001");
        assert_eq!(task.target, "192.168.1.1");
        assert_eq!(task.interval_seconds, 300);
        assert_eq!(task.priority, 100);
        assert!(task.enabled);
        assert_eq!(task.consecutive_failures, 0);
    }

    #[test]
    fn test_polling_tasks_model_disabled() {
        let task = Model {
            id: "task-002".to_string(),
            node_id: "node-002".to_string(),
            target: "192.168.1.2".to_string(),
            oids: r#"["1.3.6.1.2.1.1.3.0"]"#.to_string(),
            interval_seconds: 600,
            session_config: r#"{"community": "private", "version": "v2c"}"#.to_string(),
            priority: 50,
            enabled: false,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            last_success: None,
            last_error: Some("Authentication failed".to_string()),
            consecutive_failures: 5,
        };

        assert!(!task.enabled);
        assert_eq!(task.consecutive_failures, 5);
        assert_eq!(task.last_error, Some("Authentication failed".to_string()));
    }

    #[test]
    fn test_polling_tasks_model_serialization() {
        let task = Model {
            id: "task-001".to_string(),
            node_id: "node-001".to_string(),
            target: "192.168.1.1".to_string(),
            oids: r#"["1.3.6.1.2.1.1.3.0"]"#.to_string(),
            interval_seconds: 300,
            session_config: r#"{"community": "public", "version": "v2c"}"#.to_string(),
            priority: 100,
            enabled: true,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            last_success: None,
            last_error: None,
            consecutive_failures: 0,
        };

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(task, deserialized);
    }

    #[test]
    fn test_polling_tasks_relation_definitions() {
        let node_rel = Relation::Node.def();
        assert_eq!(node_rel.from_tbl, Entity.table_ref());
    }

    #[test]
    fn test_polling_tasks_relation_enum_iter() {
        use sea_orm::Iterable;
        let relations: Vec<Relation> = Relation::iter().collect();
        assert_eq!(relations.len(), 1);
        // Test that we can iterate over relations
        for relation in relations {
            let _rel_def = relation.def();
        }
    }
}
