use crate::parser::{data_reader::Data, inmediate_relation_reader::InmediateRelation};
#[derive(Debug, Clone)]
pub struct Truth {
    data: Vec<Data>,
}

impl Truth {
    pub fn afirms(&self, query: &Vec<Data>) -> bool {
        query == &self.data
    }
    pub fn get_data(&self) -> &Vec<Data> {
        &self.data
    }
}

impl From<Vec<Data>> for Truth {
    fn from(value: Vec<Data>) -> Self {
        Self { data: value }
    }
}

impl From<InmediateRelation> for Truth {
    fn from(value: InmediateRelation) -> Self {
        Self { data: value.args }
    }
}
