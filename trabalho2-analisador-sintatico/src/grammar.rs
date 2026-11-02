use std::collections::{BTreeMap, BTreeSet};

pub const EPSILON: &str = "ε";

#[derive(Debug, Clone)]
pub struct Production {
    pub left: String,
    pub right: Vec<String>,
}

#[derive(Debug)]
pub struct Grammar {
    pub start: String,
    pub productions: Vec<Production>,
    pub nonterminals: BTreeSet<String>,
    pub terminals: BTreeSet<String>,
    pub first: BTreeMap<String, BTreeSet<String>>,
    pub follow: BTreeMap<String, BTreeSet<String>>,
    pub table: BTreeMap<(String, String), usize>,
}

impl Grammar {
    /// Le as producoes, calcula FIRST/FOLLOW e constroi M automaticamente.
    pub fn java_subset() -> Result<Self, String> {
        let rules = [
            (
                "PROG",
                "public class id { public static void id ( ) { CMDS } }",
            ),
            ("CMDS", "CMD CMDS"),
            ("CMDS", EPSILON),
            ("CMD", "DECL"),
            ("CMD", "ATRIB"),
            ("CMD", "IF_STMT"),
            ("DECL", "TIPO id ;"),
            ("TIPO", "int"),
            ("TIPO", "float"),
            ("TIPO", "char"),
            ("ATRIB", "id = E ;"),
            ("IF_STMT", "if ( COND ) { CMDS } SENAO"),
            ("SENAO", "else { CMDS }"),
            ("SENAO", EPSILON),
            ("COND", "E OP_REL E"),
            ("OP_REL", "=="),
            ("OP_REL", "!="),
            ("OP_REL", "<"),
            ("OP_REL", ">"),
            ("E", "T E'"),
            ("E'", "+ T E'"),
            ("E'", "- T E'"),
            ("E'", EPSILON),
            ("T", "F T'"),
            ("T'", "* F T'"),
            ("T'", "/ F T'"),
            ("T'", EPSILON),
            ("F", "( E )"),
            ("F", "id"),
            ("F", "num"),
        ];
        Self::from_rules("PROG", &rules)
    }

    pub fn from_rules(start: &str, rules: &[(&str, &str)]) -> Result<Self, String> {
        let nonterminals: BTreeSet<_> = rules.iter().map(|(left, _)| (*left).to_string()).collect();
        let productions: Vec<_> = rules
            .iter()
            .map(|(left, right)| Production {
                left: (*left).to_string(),
                right: right.split_whitespace().map(str::to_string).collect(),
            })
            .collect();
        let terminals = productions
            .iter()
            .flat_map(|p| p.right.iter())
            .filter(|s| *s != EPSILON && !nonterminals.contains(*s))
            .cloned()
            .chain(["$".into()])
            .collect();
        let mut grammar = Self {
            start: start.into(),
            productions,
            nonterminals,
            terminals,
            first: BTreeMap::new(),
            follow: BTreeMap::new(),
            table: BTreeMap::new(),
        };
        grammar.compute_first();
        grammar.compute_follow();
        grammar.build_table()?;
        Ok(grammar)
    }

    fn compute_first(&mut self) {
        self.first = self
            .nonterminals
            .iter()
            .map(|nt| (nt.clone(), BTreeSet::new()))
            .collect();
        loop {
            let mut changed = false;
            for p in &self.productions {
                let seq_first =
                    Self::first_of_sequence_from(&p.right, &self.nonterminals, &self.first);
                let target = self.first.get_mut(&p.left).unwrap();
                let before = target.len();
                target.extend(seq_first);
                changed |= target.len() != before;
            }
            if !changed {
                break;
            }
        }
    }

    fn compute_follow(&mut self) {
        self.follow = self
            .nonterminals
            .iter()
            .map(|nt| (nt.clone(), BTreeSet::new()))
            .collect();
        self.follow.get_mut(&self.start).unwrap().insert("$".into());
        loop {
            let mut changed = false;
            for p in &self.productions {
                for i in 0..p.right.len() {
                    let symbol = &p.right[i];
                    if !self.nonterminals.contains(symbol) {
                        continue;
                    }
                    let suffix_first = Self::first_of_sequence_from(
                        &p.right[i + 1..],
                        &self.nonterminals,
                        &self.first,
                    );
                    let mut additions: BTreeSet<_> = suffix_first
                        .iter()
                        .filter(|s| s.as_str() != EPSILON)
                        .cloned()
                        .collect();
                    if i + 1 == p.right.len() || suffix_first.contains(EPSILON) {
                        additions.extend(self.follow[&p.left].iter().cloned());
                    }
                    let target = self.follow.get_mut(symbol).unwrap();
                    let before = target.len();
                    target.extend(additions);
                    changed |= target.len() != before;
                }
            }
            if !changed {
                break;
            }
        }
    }

    fn build_table(&mut self) -> Result<(), String> {
        for (index, p) in self.productions.iter().enumerate() {
            let first_alpha = self.first_of_sequence(&p.right);
            let mut columns: BTreeSet<_> = first_alpha
                .iter()
                .filter(|s| s.as_str() != EPSILON)
                .cloned()
                .collect();
            if first_alpha.contains(EPSILON) {
                columns.extend(self.follow[&p.left].iter().cloned());
            }
            for terminal in columns {
                let key = (p.left.clone(), terminal.clone());
                if let Some(old) = self.table.insert(key, index) {
                    return Err(format!(
                        "conflito LL(1) em M[{}, {}]: producoes {} e {}",
                        p.left,
                        terminal,
                        old + 1,
                        index + 1
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn first_of_sequence(&self, sequence: &[String]) -> BTreeSet<String> {
        Self::first_of_sequence_from(sequence, &self.nonterminals, &self.first)
    }

    fn first_of_sequence_from(
        sequence: &[String],
        nonterminals: &BTreeSet<String>,
        first: &BTreeMap<String, BTreeSet<String>>,
    ) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        if sequence.is_empty() {
            result.insert(EPSILON.into());
            return result;
        }
        let mut nullable_prefix = true;
        for symbol in sequence {
            if symbol == EPSILON {
                result.insert(EPSILON.into());
                break;
            }
            if !nonterminals.contains(symbol) {
                result.insert(symbol.clone());
                nullable_prefix = false;
                break;
            }
            let set = &first[symbol];
            result.extend(set.iter().filter(|s| s.as_str() != EPSILON).cloned());
            if !set.contains(EPSILON) {
                nullable_prefix = false;
                break;
            }
        }
        if nullable_prefix {
            result.insert(EPSILON.into());
        }
        result
    }

    pub fn expected(&self, nonterminal: &str) -> Vec<String> {
        self.table
            .keys()
            .filter(|(nt, _)| nt == nonterminal)
            .map(|(_, t)| t.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_representative_first_and_follow_sets() {
        let g = Grammar::java_subset().unwrap();
        assert_eq!(
            g.first["F"],
            ["(", "id", "num"].into_iter().map(str::to_string).collect()
        );
        assert!(g.first["CMDS"].contains(EPSILON));
        assert!(g.follow["E"].contains(")"));
        assert!(g.follow["E"].contains(";"));
    }

    #[test]
    fn generated_grammar_has_no_ll1_conflicts() {
        let g = Grammar::java_subset().unwrap();
        assert_eq!(
            g.table.get(&("TIPO".into(), "int".into())).copied(),
            Some(7)
        );
        assert!(g.table.contains_key(&("E'".into(), ";".into())));
    }
}
