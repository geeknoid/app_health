#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    All,
    OnlyNominal,
    OnlyDegraded,
    OnlyCritical,
    OnlyNonFunctional,
    AllNonNominal,
}
