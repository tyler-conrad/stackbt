#[cfg(feature = "try_trait")]
use std::ops::Try;

/// A generic enum which are provided to help implementations of certain 
/// behavior tree nodes choose whether a particular state is nonterminal or 
/// terminal, and to work with nonterminal or terminal states their children 
/// have themselves chosen. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Statepoint<N, T> {
    /// A nonterminal state. 
    Nonterminal(N),
    /// A terminal state. 
    Terminal(T),
}

#[cfg(feature = "try_trait")]
impl<N, T> Try for Statepoint<N, T> {
    type Ok = N;
    type Error = T;

    fn into_result(self) -> Result<N, T> {
        match self {
            Statepoint::Nonterminal(n) => Result::Ok(n),
            Statepoint::Terminal(t) => Result::Err(t)
        }
    }

    fn from_error(term: T) -> Self {
        Statepoint::Terminal(term)
    }

    fn from_ok(nonterm: N) -> Self {
        Statepoint::Nonterminal(nonterm)
    }
}

/// The return value of behavior tree nodes. To statically prevent further 
/// running after a node reaches a terminal state, the whole node is taken by 
/// move during a step. At a nonterminal, the nonterminal decision point value 
/// is returned along with the modified behavior tree node, while at a terminal, 
/// only the terminal decision point value is returned, with the node instance 
/// dropped and never to return. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NodeResult<R, T, N> {
    /// A nonterminal state, along with the node itself. 
    Nonterminal(R, N),
    /// A terminal state. 
    Terminal(T)
}

#[cfg(feature = "try_trait")]
impl<R, T, N> Try for NodeResult<R, T, N> {
    type Ok = (R, N);
    type Error = T;

    fn into_result(self) -> Result<(R, N), T> {
        match self {
            NodeResult::Nonterminal(r, n) => Result::Ok((r, n)),
            NodeResult::Terminal(t) => Result::Err(t)
        }
    }

    fn from_error(term: T) -> Self {
        NodeResult::Terminal(term)
    }

    fn from_ok(nonterm: (R, N)) -> Self {
        NodeResult::Nonterminal(nonterm.0, nonterm.1)
    }
}

/// The behavior tree node trait itself. 
pub trait BehaviorTreeNode {
    /// Type of the input to take. 
    type Input;
    /// Type of the nonterminal statepoints returned. 
    type Nonterminal;
    /// Type of the terminal statepoints returned. 
    type Terminal;

    /// Given the input, perform a single step of the behavior node, 
    /// either returning itself along with a nonterminal state, or returning 
    /// a terminal state. 
    fn step(self, input: &Self::Input) -> 
        NodeResult<Self::Nonterminal, Self::Terminal, Self> where 
        Self: Sized;
}

#[cfg(all(test, feature = "try_trait"))]
mod tests_try {
    use std::ops::Try;

    #[test]
    fn statepoint_try_test() {
        use behavior_tree_node::Statepoint;
        assert_eq!(Statepoint::Nonterminal::<i64, i64>(5).into_result(), 
            Result::Ok(5));
        assert_eq!(Statepoint::Terminal::<i64, i64>(5).into_result(), 
            Result::Err(5));
        assert_eq!(Statepoint::<i64, i64>::from_error(5), Statepoint::Terminal(5));
        assert_eq!(Statepoint::<i64, i64>::from_ok(5), Statepoint::Nonterminal(5));
    }
    #[test]
    fn node_result_try_test() {
        use behavior_tree_node::NodeResult;
        assert_eq!(NodeResult::Nonterminal::<i64, i64, i64>(5, 4).into_result(), 
            Result::Ok((5, 4)));
        assert_eq!(NodeResult::Terminal::<i64, i64, i64>(5).into_result(), 
            Result::Err(5));
        assert_eq!(NodeResult::<i64, i64, i64>::from_error(5), 
            NodeResult::Terminal(5));
        assert_eq!(NodeResult::<i64, i64, i64>::from_ok((5, 4)), 
            NodeResult::Nonterminal(5, 4));
    }
}