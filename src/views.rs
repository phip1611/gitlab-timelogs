/*
MIT License

Copyright (c) 2024 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Provides transform functions for different views into the data.

use crate::gitlab_api::types::{Epic, Issue, ResponseNode};
use chrono::{Datelike, IsoWeek, NaiveDate};
use std::collections::BTreeMap;
use std::time::Duration;

fn group_notes_by_filter<'a, T: PartialEq + Ord>(
    nodes: &[&'a ResponseNode],
    map_fn: impl Fn(&ResponseNode) -> T,
) -> BTreeMap<T, Vec<&'a ResponseNode>> {
    let items = nodes.iter().map(|node| map_fn(node)).collect::<Vec<_>>();

    let mut map = BTreeMap::new();
    for item in items {
        let nodes_of_week = nodes
            .iter()
            .filter(|node| map_fn(node) == item)
            .cloned()
            .collect::<Vec<_>>();

        map.entry(item).or_insert(nodes_of_week);
    }
    map
}

/// Returns the nodes per [`IsoWeek`].
pub fn to_nodes_by_week<'a>(
    nodes: &[&'a ResponseNode],
) -> BTreeMap<IsoWeek, Vec<&'a ResponseNode>> {
    group_notes_by_filter(nodes, |node| node.datetime().iso_week())
}

/// Returns the nodes per [`NaiveDate`].
pub fn to_nodes_by_day<'a>(
    nodes: &[&'a ResponseNode],
) -> BTreeMap<NaiveDate, Vec<&'a ResponseNode>> {
    group_notes_by_filter(nodes, |node| node.datetime())
}

/// Returns the nodes per [`Epic`].
pub fn to_nodes_by_epic<'a>(
    nodes: &[&'a ResponseNode],
) -> BTreeMap<Option<Epic>, Vec<&'a ResponseNode>> {
    group_notes_by_filter(nodes, |node| node.issue.epic.clone())
}

/// Returns the nodes per [`Issue`].
pub fn to_nodes_by_issue<'a>(nodes: &[&'a ResponseNode]) -> BTreeMap<Issue, Vec<&'a ResponseNode>> {
    group_notes_by_filter(nodes, |node| node.issue.clone())
}

/// Returns the time spent per day.
pub fn to_time_spent_sum(nodes: &[&ResponseNode]) -> Duration {
    nodes.iter().map(|node| node.timeSpent().1).sum()
}
