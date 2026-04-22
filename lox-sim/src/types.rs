/// All connectors carry f64. Digital = 0.0/1.0, Analog = arbitrary f64.
pub type Signal = f64;

/// Unique identifier for a connector (index into global signal array).
pub type ConnectorId = usize;

/// Unique identifier for a block.
pub type BlockId = usize;

/// Connector direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectorDir {
    Input,
    Output,
    Parameter,
}

/// Connector metadata.
#[derive(Debug, Clone)]
pub struct ConnectorInfo {
    pub id: ConnectorId,
    pub key: String,
    pub dir: ConnectorDir,
    pub block_id: BlockId,
    pub default_value: Signal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_dir_equality() {
        assert_eq!(ConnectorDir::Input, ConnectorDir::Input);
        assert_ne!(ConnectorDir::Input, ConnectorDir::Output);
        assert_ne!(ConnectorDir::Output, ConnectorDir::Parameter);
    }

    #[test]
    fn connector_dir_copy() {
        let a = ConnectorDir::Input;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn connector_info_construction() {
        let info = ConnectorInfo {
            id: 42,
            key: "AQ1".to_string(),
            dir: ConnectorDir::Output,
            block_id: 7,
            default_value: 3.14,
        };
        assert_eq!(info.id, 42);
        assert_eq!(info.key, "AQ1");
        assert_eq!(info.dir, ConnectorDir::Output);
        assert_eq!(info.block_id, 7);
        assert!((info.default_value - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn connector_info_clone() {
        let info = ConnectorInfo {
            id: 0,
            key: "I1".to_string(),
            dir: ConnectorDir::Input,
            block_id: 0,
            default_value: 0.0,
        };
        let cloned = info.clone();
        assert_eq!(cloned.id, info.id);
        assert_eq!(cloned.key, info.key);
        assert_eq!(cloned.dir, info.dir);
    }

    #[test]
    fn signal_is_f64() {
        let s: Signal = 3.14;
        assert!((s - std::f64::consts::PI).abs() < 0.01);
    }

    #[test]
    fn connector_id_arithmetic() {
        let a: ConnectorId = 5;
        let b: ConnectorId = 3;
        assert_eq!(a + b, 8);
        assert_eq!(a - b, 2);
    }

    #[test]
    fn block_id_is_usize() {
        let id: BlockId = usize::MAX;
        assert_eq!(id, usize::MAX);
    }
}
