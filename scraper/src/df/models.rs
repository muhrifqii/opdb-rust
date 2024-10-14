use super::df_type::DfType;

#[derive(Debug)]
pub struct DfTypeInfo {
    pub df_type: DfType,
    pub cannon_count: u32,
    pub non_cannon_count: u32,
    pub description: String,
}

impl std::fmt::Display for DfTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(df_type: {}, cannon: {}, non-cannon: {}, description: {})",
            self.df_type, self.cannon_count, self.non_cannon_count, self.description
        )
    }
}
