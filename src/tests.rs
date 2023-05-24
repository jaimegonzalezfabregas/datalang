#[cfg(test)]
mod tests {
    use crate::engine::Engine;

    #[test]
    fn read_inmediate_relation() {
        let mut engine = Engine::new();
        engine.input("rel(0,1)".into(), false);
        assert_eq!(
            "Engine { tables: {RelId { identifier: \"rel\", column_count: 2 }: Table { width: 2, history: [IsFalseThat([Number(0.0), Number(1.0)])] }} }",
            format!("{engine:?}")
        );
    }

    #[test]
    fn read_defered_relation() {
        let mut engine = Engine::new();
        assert_eq!("", engine.input("rel(0,1) rel(_,_)?".into(), false));
    }
}
