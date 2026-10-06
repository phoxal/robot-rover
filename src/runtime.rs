//! Empty mandatory brain for the basic four-wheel composition.

#[phoxal::endpoints]
pub(crate) struct BrainApi {}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Brain;

#[phoxal::runtime(
    contract = BrainApi,
    period_ms = 20,
    timeout_ms = 100,
    init_timeout_ms = 1_000
)]
impl Brain {
    #[init]
    fn new(_config: ()) -> phoxal::Result<Self> {
        Ok(Self)
    }
}
