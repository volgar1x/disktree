use std::cmp::Ordering;

use super::ItemStore;
use crate::KeyRef;

#[derive(Clone, Copy, Debug)]
pub enum SearchResult {
    Found { index: usize, group_index: usize },
    GroupFound { index: usize, group_index: usize },
    NotFound { index: usize },
}

impl SearchResult {
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found { .. })
    }
}

impl ItemStore<'_> {
    pub(crate) fn binary_search(
        &self,
        key: impl AsRef<KeyRef>,
        start: Option<(usize, usize)>,
    ) -> SearchResult {
        let key = key.as_ref();
        let cur_start = start.map_or(0, |start| start.0);
        let mut cur = cur_start..self.0.len();
        while !cur.is_empty() {
            let index = (cur.start + cur.end) / 2;

            let group = &self.0[index];
            let group_start = start
                .filter(|start| start.0 == index)
                .map_or(0, |start| start.1);

            match key.cmp(group[group_start].key()) {
                Ordering::Equal => {
                    return SearchResult::Found {
                        index,
                        group_index: group_start,
                    };
                }
                Ordering::Less => {
                    cur = cur.start..index;
                }
                Ordering::Greater => match key.cmp(group[group.len() - 1].key()) {
                    Ordering::Equal => {
                        return SearchResult::Found {
                            index,
                            group_index: group.len() - 1,
                        };
                    }
                    Ordering::Less => {
                        match group[group_start..].binary_search_by(|item| item.key().cmp(key)) {
                            Ok(group_index) => {
                                return SearchResult::Found {
                                    index,
                                    group_index: group_start + group_index,
                                };
                            }
                            Err(group_index) => {
                                return SearchResult::GroupFound {
                                    index,
                                    group_index: group_start + group_index,
                                };
                            }
                        }
                    }
                    Ordering::Greater => {
                        cur = (index + 1)..cur.end;
                    }
                },
            }
        }

        SearchResult::NotFound { index: cur.start }
    }
}
