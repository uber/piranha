use crate::df::basic_analysis::ForwardReachingDirection;
use crate::df::df::DataflowAnalysis;

#[cfg(test)]
fn test() {
    let mut analysis = DataflowAnalysis::new(ForwardReachingDirection);
    analysis.run_analysis(vec![], ());
}
