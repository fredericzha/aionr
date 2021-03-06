/*******************************************************************************
 * Copyright (c) 2015-2018 Parity Technologies (UK) Ltd.
 * Copyright (c) 2018-2019 Aion foundation.
 *
 *     This file is part of the aion network project.
 *
 *     The aion network project is free software: you can redistribute it
 *     and/or modify it under the terms of the GNU General Public License
 *     as published by the Free Software Foundation, either version 3 of
 *     the License, or any later version.
 *
 *     The aion network project is distributed in the hope that it will
 *     be useful, but WITHOUT ANY WARRANTY; without even the implied
 *     warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 *     See the GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with the aion network project source files.
 *     If not, see <https://www.gnu.org/licenses/>.
 *
 ******************************************************************************/

//! Blockchain filter

use aion_types::{H256, Address};
use ethbloom::{Bloom, Input as BloomInput};
use ids::BlockId;
use log_entry::LogEntry;

/// Blockchain Filter.
#[derive(Debug, PartialEq)]
pub struct Filter {
    /// Blockchain will be searched from this block.
    pub from_block: BlockId,

    /// Till this block.
    pub to_block: BlockId,

    /// Search addresses.
    ///
    /// If None, match all.
    /// If specified, log must be produced by one of these addresses.
    pub address: Option<Vec<Address>>,

    /// Search topics.
    ///
    /// If None, match all.
    /// If specified, log must contain one of these topics.
    pub topics: Vec<Option<Vec<H256>>>,

    /// Logs limit
    ///
    /// If None, return all logs
    /// If specified, should only return *last* `n` logs.
    pub limit: Option<usize>,
}

impl Clone for Filter {
    fn clone(&self) -> Self {
        let mut topics = [None, None, None, None];
        for i in 0..4 {
            topics[i] = self.topics[i].clone();
        }

        Filter {
            from_block: self.from_block.clone(),
            to_block: self.to_block.clone(),
            address: self.address.clone(),
            topics: topics[..].to_vec(),
            limit: self.limit,
        }
    }
}

impl Filter {
    /// Returns combinations of each address and topic.
    pub fn bloom_possibilities(&self) -> Vec<Bloom> {
        let blooms = match self.address {
            Some(ref addresses) if !addresses.is_empty() => {
                addresses
                    .iter()
                    .map(|ref address| Bloom::from(BloomInput::Raw(address)))
                    .collect()
            }
            _ => vec![Bloom::default()],
        };

        self.topics.iter().fold(blooms, |bs, topic| {
            match *topic {
                None => bs,
                Some(ref topics) => {
                    bs.into_iter()
                        .flat_map(|bloom| {
                            topics
                                .into_iter()
                                .map(|topic| {
                                    let mut b = bloom.clone();
                                    b.accrue(BloomInput::Raw(topic));
                                    b
                                })
                                .collect::<Vec<Bloom>>()
                        })
                        .collect()
                }
            }
        })
    }

    /// Returns true if given log entry matches filter.
    pub fn matches(&self, log: &LogEntry) -> bool {
        let matches = match self.address {
            Some(ref addresses) if !addresses.is_empty() => {
                addresses.iter().any(|address| &log.address == address)
            }
            _ => true,
        };

        matches && self.topics.iter().enumerate().all(|(i, topic)| {
            match *topic {
                Some(ref topics) if !topics.is_empty() => {
                    topics.iter().any(|topic| log.topics.get(i) == Some(topic))
                }
                _ => true,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use ethbloom::Bloom;
    use filter::Filter;
    use ids::BlockId;
    use log_entry::LogEntry;

    #[test]
    fn test_bloom_possibilities_none() {
        let none_filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: None,
            topics: vec![None, None, None, None],
            limit: None,
        };

        let possibilities = none_filter.bloom_possibilities();
        assert_eq!(possibilities.len(), 1);
        assert!(possibilities[0].is_zero())
    }

    // block 399849
    #[test]
    fn test_bloom_possibilities_single_address_and_topic() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![
                "a00a2d0d10ce8a2ea47a76fbb935405df2a12b0e2bc932f188f84b5f16da9c2c".into(),
            ]),
            topics: vec![
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                ]),
                None,
                None,
                None,
            ],
            limit: None,
        };

        let possibilities = filter.bloom_possibilities();
        assert_eq!(possibilities, vec!["0x00000000000000000000000008000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000040".into()] as Vec<Bloom>);
    }

    #[test]
    fn test_bloom_possibilities_single_address_and_many_topics() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![
                "a00a2d0d10ce8a2ea47a76fbb935405df2a12b0e2bc932f188f84b5f16da9c2c".into(),
            ]),
            topics: vec![
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                ]),
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                ]),
                None,
                None,
            ],
            limit: None,
        };

        let possibilities = filter.bloom_possibilities();
        assert_eq!(possibilities, vec!["0x00000000000000000000000008000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000040".into()] as Vec<Bloom>);
    }

    #[test]
    fn test_bloom_possibilites_multiple_addresses_and_topics() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![
                "a00a2d0d10ce8a2ea47a76fbb935405df2a12b0e2bc932f188f84b5f16da9c2c".into(),
                "a02c25e39471085ff8cae0882132d82c6490eb02f3e6906b303d8f990e86e634".into(),
            ]),
            topics: vec![
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                ]),
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                ]),
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                ]),
                None,
            ],
            limit: None,
        };

        // number of possibilites should be equal 2 * 2 * 2 * 1 = 8
        let possibilities = filter.bloom_possibilities();
        assert_eq!(possibilities.len(), 8);
        assert_eq!(possibilities[0], Bloom::from("0x00000000000000000000000008000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000040"));
    }

    #[test]
    fn test_filter_matches() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![
                "a00a2d0d10ce8a2ea47a76fbb935405df2a12b0e2bc932f188f84b5f16da9c2c".into(),
            ]),
            topics: vec![
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                ]),
                Some(vec![
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa".into(),
                ]),
                None,
                None,
            ],
            limit: None,
        };

        let entry0 = LogEntry {
            address: "a00a2d0d10ce8a2ea47a76fbb935405df2a12b0e2bc932f188f84b5f16da9c2c".into(),
            topics: vec![
                "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa".into(),
                "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
            ],
            data: vec![],
        };

        let entry1 = LogEntry {
            address: "a02c25e39471085ff8cae0882132d82c6490eb02f3e6906b303d8f990e86e634".into(),
            topics: vec![
                "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
                "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa".into(),
                "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into(),
            ],
            data: vec![],
        };

        let entry2 = LogEntry {
            address: "a00a2d0d10ce8a2ea47a76fbb935405df2a12b0e2bc932f188f84b5f16da9c2c".into(),
            topics: vec!["ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9".into()],
            data: vec![],
        };

        assert_eq!(filter.matches(&entry0), true);
        assert_eq!(filter.matches(&entry1), false);
        assert_eq!(filter.matches(&entry2), false);
    }
}
