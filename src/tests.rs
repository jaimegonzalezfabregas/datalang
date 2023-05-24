#[cfg(test)]
mod tests {
    use crate::engine::Engine;

    #[test]
    fn read_inmediate_relation_add() {
        let mut engine = Engine::new();
        engine.input("rel(0,1)".into(), true);
        assert_eq!(
            "Engine { tables: {RelId { identifier: \"rel\", column_count: 2 }: Table { rel_id: RelId { identifier: \"rel\", column_count: 2 }, history: [IsTrueThat(Truth { data: [Number(0.0), Number(1.0)] })] }} }",
            format!("{engine:?}")
        );
    }

    #[test]
    fn read_inmediate_relation_substract() {
        let mut engine = Engine::new();
        engine.input("rel(0,1) !rel(0,1)".into(), true);
        assert_eq!(
            "Engine { tables: {RelId { identifier: \"rel\", column_count: 2 }: Table { rel_id: RelId { identifier: \"rel\", column_count: 2 }, history: [] }} }",
            format!("{engine:?}")
        );
    }

    #[test]
    fn query_full_table_1() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(0.0), Number(1.0)] }]",
            engine.input("rel(0,1) rel(_,_)?".into(), true)
        );
    }

    #[test]
    fn query_full_table_2() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(0.0), Number(1.0)] }, Truth { data: [String(\"hola\"), Number(1.0)] }]",
            engine.input("rel(0,1) rel(\"hola\",1) rel(_,_)?".into(), true)
        );
    }

    #[test]
    fn query_filter_table_2() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [String(\"filtro\"), Number(1.0)] }]",
            engine.input(
                "rel(\"clave\",1) rel(\"filtro\",1) rel(\"filtro\",_)?".into(),
                true
            )
        );
    }

    #[test]
    fn view_1() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(0.0)] }, Truth { data: [Number(1.0)] }, Truth { data: [Number(2.0)] }, Truth { data: [Number(3.0)] }]",
            engine.input("rel(0,1) rel(2,3) test(a) :- rel(a,_) || rel(_,a) test(_)?".into(), true)
        );
    }

    #[test]
    fn view_2() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(3.0)] }, Truth { data: [Number(4.0)] }]",
            engine.input("rel(4,4) rel(0,1) rel(2,3) rel(2,2) !rel(2,2) rel(3,3) test(a) :- rel(a,a) test(_)?".into(), true)
        );
    }

    #[test]
    fn view_resolving_verbose_1() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input(
                "rel(0) relSuc(suc) :- rel(a) && a = suc-1 relSuc(_)?".into(),
                true
            )
        );
    }

    #[test]
    fn view_resolving_verbose_2() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input(
                "rel(0) relSuc(suc) :- rel(a) && a+1 = suc relSuc(_)?".into(),
                true
            )
        );
    }

    fn view_resolving_verbose_1_reverse() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input(
                "rel(0) relSuc(suc) :- a = suc-1 && rel(a) relSuc(_)?".into(),
                true
            )
        );
    }

    #[test]
    fn view_resolving_verbose_2_reverse() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input(
                "rel(0) relSuc(suc) :- a+1 = suc && rel(a) relSuc(_)?".into(),
                true
            )
        );
    }

    #[test]
    fn view_resolving_1() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input("rel(0) relSuc(suc) :- rel(suc-1) relSuc(_)?".into(), true)
        );
    }

    #[test]
    fn view_resolving_2() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input("rel(0) relSuc(a+1) :- rel(a) relSuc(_)?".into(), true)
        );
    }

    #[test]
    fn view_resolving_3() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input(
                "rel1(0) rel1(1) rel2(1) rel2(2) test(a) :- rel1(a) && rel2(a) test(_)?".into(),
                true
            )
        );
    }

    #[test]
    fn view_resolving_4() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input(
                "rel1(0) rel1(1) rel2(1) rel2(2) test(a) :- rel1(b) && rel2(c) && b=c && a=b test(_)?"
                    .into(),
                true
            )
        );
    }

    #[test]
    fn equation_resolving() {
        let mut engine = Engine::new();
        assert_eq!(
            "[Truth { data: [Number(1.0)] }]",
            engine.input("relSuc(suc) :- 0 = suc - 1 relSuc(_)?".into(), true)
        );
    }
}
