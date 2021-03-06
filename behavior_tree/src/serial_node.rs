use behavior_tree_node::{BehaviorTreeNode, NodeResult};
use num_traits::FromPrimitive;


/// Trait for an enumeration of nodes, all of which have the same input, 
/// nonterminals, and terminals. Each variant corresponds to a different 
/// possible subnode of the enumerable supernode. 
pub trait EnumNode: BehaviorTreeNode {
    /// The type used to enumerate the variants of implementations of this 
    /// trait. std::mem::Discriminant works for comparing variants of an enum,
    /// but not for enumerating or matching against them, hence this 
    /// associated type. 
    type Discriminant: Copy;

    /// Initialize a new node with the given discriminant value. 
    fn new(Self::Discriminant) -> Self;

    fn discriminant_of(&self) -> Self::Discriminant;
}

/// Declarative macro for quickly and easily declaring an serial node enum.
#[cfg(feature = "existential_type")]
#[macro_export]
macro_rules! enum_node {
    (
        type Input = $inputtype:ty ;
        type Nonterminal = $nontermtype:ty ;
        type Terminal = $termtype:ty ;
        $( #[ $mval:meta ] )*
        enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident ( $( $statements:stmt )* )
            ),*
        }
    ) => {
        $(
            existential type $variant : BehaviorTreeNode<Input = $inputtype,
                Nonterminal = $nontermtype, Terminal = $termtype > ;
        )*

        $( #[ $mval ] )*
        enum $name {
            $(
                $( #[ $emval ] )*
                $variant ( $variant )
            ),*
        }

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[derive(::num_derive::ToPrimitive, ::num_derive::FromPrimitive)]
        enum $itername {
            $( $variant ),*
        }

        impl BehaviorTreeNode for $name {
            type Input = $inputtype;
            type Nonterminal = $nontermtype;
            type Terminal = $termtype;

            fn step(self, input: & $inputtype) -> NodeResult< $nontermtype , 
                $termtype , Self > where Self: Sized 
            {
                match self {
                    $(
                        $name :: $variant (val) => match val.step(input) {
                            NodeResult::Nonterminal(v, o) => NodeResult::Nonterminal(
                                v, 
                                $name :: $variant (o)
                            ),
                            NodeResult::Terminal(v) => NodeResult::Terminal(v)
                        }
                    ),*
                }
            }
        }

        impl EnumNode for $name {
            type Discriminant = $itername;

            fn new(discriminant: $itername) -> Self {
                match discriminant {
                    $(
                        $itername :: $variant => $name :: $variant ( 
                            (| | -> $variant { $( $statements )* })()
                        )
                    ),*
                }
            }

            fn discriminant_of(&self) -> $itername {
                match self {
                    $( $name :: $variant (_) => $itername :: $variant ),*
                }
            }
        }
    };
}

/// Enumeration of the possible decisions when the child node reaches a 
/// nonterminal state. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NontermDecision<E, T, X> {
    /// Step the current subnode. 
    Step(T),
    /// Transition from the current subnode to a new one. 
    Trans(E, T),
    /// Exit the current supernode entirely. 
    Exit(X)
}

/// Enumeration of the possible decisions when the child node reaches a 
/// terminal state. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TermDecision<E, T, X> {
    /// Transition from the current subnode to a new one. 
    Trans(E, T),
    /// Exit the current supernode entirely. 
    Exit(X)
}

/// Return type of the SerialBranchNode. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NontermReturn<E, N, T> {
    /// Nonterminal of a subnode. 
    Nonterminal(E, N),
    /// Terminal of a subnode. 
    Terminal(E, T)
}

/// Trait for the transition behavior of a SerialBranchNode. 
pub trait SerialDecider {
    /// Type of the enumerating discriminant
    type Enum;
    /// Type of the inputs of the subnodes. 
    type Input;
    /// Type of the nonterminals of the subnodes. 
    type Nonterm;
    /// Type of the terminals of the subnodes. 
    type Term;
    /// Supernode terminal type. 
    type Exit;
    /// Given a reference to the input and the current nonterminal state, 
    /// decide what to do from the nonterminal statepoint. 
    fn on_nonterminal(&self, &Self::Input, Self::Enum, Self::Nonterm) -> NontermDecision<
        Self::Enum, Self::Nonterm, Self::Exit>;
    /// Given a reference to the input and the current terminal state, decide 
    /// what to do from the terminal statepoint. 
    fn on_terminal(&self, &Self::Input, Self::Enum, Self::Term) -> TermDecision<
        Self::Enum, Self::Term, Self::Exit>;
}

/// A serial branch node, which is composed of a SerialDecider on top of a 
/// special enumerable node type. 
/// 
/// The idea behind this node is that the EnumNode trait describes the 
/// possible subordinate nodes of this node, and that execution proceeds along
/// one, before a new child node is switched to based on the current state and 
/// the input, along which execution subsequently proceeds, and after some 
/// time, a new node may be switched to or the whole parent node transitioned 
/// from. 
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SerialBranchNode<E, D> where
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal>
{
    node: E,
    decider: D
}

impl<E, D> SerialBranchNode<E, D> where 
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal>
{
    /// Create a new serial branch node for the given discriminant. 
    pub fn new(decider: D, variant: E::Discriminant) -> SerialBranchNode<E, D> {
        SerialBranchNode {
            node: E::new(variant),
            decider: decider
        }

    }

    /// Wrap an existing enumerated node in a serial branch node. 
    pub fn from_existing(decider: D, existing: E) -> SerialBranchNode<E, D> {
        SerialBranchNode {
            node: existing,
            decider: decider
        }
    }
}

impl<E, D> Default for SerialBranchNode<E, D> where 
    E: EnumNode,
    E::Discriminant: FromPrimitive, 
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal> + Default
{
    fn default() -> SerialBranchNode<E, D> {
        SerialBranchNode::new(D::default(), E::Discriminant::from_u64(0).unwrap())
    }
}

impl<E, D> BehaviorTreeNode for SerialBranchNode<E, D> where
    E: EnumNode,
    D: SerialDecider<Enum=E::Discriminant, Input=E::Input, Nonterm=E::Nonterminal, 
        Term=E::Terminal>
{
    type Input = E::Input;
    type Nonterminal = NontermReturn<E::Discriminant, E::Nonterminal, E::Terminal>;
    type Terminal = D::Exit;

    #[inline]
    fn step(self, input: &E::Input) -> NodeResult<Self::Nonterminal, D::Exit, Self> {
        let discriminant = self.node.discriminant_of();
        match self.node.step(input) {
            NodeResult::Nonterminal(i, n) => {
                match self.decider.on_nonterminal(input, discriminant, i) {
                    NontermDecision::Step(j) => NodeResult::Nonterminal(
                        NontermReturn::Nonterminal(discriminant, j),
                        Self::from_existing(self.decider, n)
                    ),
                    NontermDecision::Trans(e, j) => NodeResult::Nonterminal(
                        NontermReturn::Nonterminal(discriminant, j),
                        Self::new(self.decider, e)
                    ),
                    NontermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            },
            NodeResult::Terminal(i) => {
                match self.decider.on_terminal(input, discriminant, i) {
                    TermDecision::Trans(e, j) => NodeResult::Nonterminal(
                        NontermReturn::Terminal(discriminant, j),
                        Self::new(self.decider, e)
                    ),
                    TermDecision::Exit(x) => NodeResult::Terminal(x)
                }
            }
        }
    }
}

#[cfg(all(test, feature = "existential_type"))]
mod tests {
    use base_nodes::{PredicateWait};
    use behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint};
    use serial_node::{EnumNode, SerialDecider, NontermDecision, TermDecision};
    use num_derive::{FromPrimitive, ToPrimitive};

    enum_node! {
        type Input = i64;
        type Nonterminal = i64;
        type Terminal = i64;

        enum MultiMachine: PosNegEnum {
            Positive (PredicateWait::new(|input: &i64| {
                if *input >= 0 {
                    Statepoint::Nonterminal(*input)
                } else {
                    Statepoint::Terminal(*input)
                }
            })),
            Negative (PredicateWait::new(|input: &i64| {
                if *input >= 0 {
                    Statepoint::Nonterminal(-*input)
                } else {
                    Statepoint::Terminal(-*input)
                }
            }))
        }
    }

    struct Switcharound;

    impl SerialDecider for Switcharound {
        type Enum = PosNegEnum;
        type Input = i64;
        type Nonterm = i64;
        type Term = i64;
        type Exit = ();
        
        fn on_nonterminal(&self, _i: &i64, _s: PosNegEnum, o: i64) -> NontermDecision<
            PosNegEnum, i64, ()> 
        {
            NontermDecision::Step(o)
        }

        fn on_terminal(&self, _i: &i64, state: PosNegEnum, o: i64) -> TermDecision<
            PosNegEnum, i64, ()> 
        {
            match state {
                PosNegEnum::Positive => TermDecision::Trans(PosNegEnum::Negative, o),
                PosNegEnum::Negative => TermDecision::Trans(PosNegEnum::Positive, o)
            }
        }
    }

    #[test]
    fn serial_switcharound_test() {
        use serial_node::{SerialBranchNode, NontermReturn};
        let test_node = SerialBranchNode::<
            MultiMachine, _>::new(Switcharound, PosNegEnum::Positive);
        let test_node_1 = match test_node.step(&5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Nonterminal(s, v) => {
                        let _: i64 = v;
                        match s {
                            PosNegEnum::Positive => (),
                            _ => unreachable!("Expected positive")
                        }
                        assert_eq!(v, 5_i64);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_2 = match test_node_1.step(&-5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Terminal(s, v) => {
                        let _: i64 = v;
                        match s {
                            PosNegEnum::Positive => (),
                            _ => unreachable!("Expected positive")
                        }
                        assert_eq!(v, -5)
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition"),
                };
                n
            },
            NodeResult::Terminal(_) => unreachable!("Expected nonterminal transition")
        };
        let test_node_3 = match test_node_2.step(&5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Nonterminal(s, v) => {
                        let _: i64 = v;
                        match s {
                            PosNegEnum::Negative => (),
                            _ => unreachable!("Expected negative"),
                        }
                        assert_eq!(v, -5)
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        let test_node_4 = match test_node_3.step(&-5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Terminal(s, v) => {
                        let _: i64 = v;
                        match s {
                            PosNegEnum::Negative => (),
                            _ => unreachable!("Expected negative"),
                        }
                        assert_eq!(v, 5)
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition"),
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
        match test_node_4.step(&5) {
            NodeResult::Nonterminal(r, n) => {
                match r {
                    NontermReturn::Nonterminal(s, v) => {
                        let _: i64 = v;
                        match s {
                            PosNegEnum::Positive => (),
                            _ => unreachable!("Expected positive")
                        }
                        assert_eq!(v, 5);
                    },
                    _ => unreachable!("Expected subordinate nonterminal transition")
                };
                n
            },
            _ => unreachable!("Expected nonterminal transition")
        };
    }

}