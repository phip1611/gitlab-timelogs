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

//! Convenience abstractions to filter the [`ResponseNode`]s of the [`Response`]
//! from the GitLab API.

use crate::gitlab_api::types::{Response, ResponseNode};
use chrono::{Datelike, NaiveDate};

/// Filters the `timelogs` [`Response`] and only emits [`ResponseNode`]s
/// matching all filters.
///
/// # Parameters
/// - `data`: The entire [`Response`]
/// - `time_filter`: The optional [`TimeFilter`] to apply
/// - `group_filter`: The optional [`GroupFilter`] to apply
/// - `epic_filter`: The optional [`EpicFilter`] to apply
pub fn filter_timelogs<'a>(
    response: &'a Response,
    time_filter: Option<TimeFilter>,
    group_filter: Option<GroupFilter<'a>>,
    epic_filter: Option<EpicFilter<'a>>,
) -> impl Iterator<Item = &'a ResponseNode> {
    response
        .data
        .timelogs
        .nodes
        .iter()
        .filter_nodes(
            time_filter
                .map(FilterKind::Time)
                .unwrap_or(FilterKind::Noop),
        )
        .filter_nodes(
            group_filter
                .map(FilterKind::Group)
                .unwrap_or(FilterKind::Noop),
        )
        .filter_nodes(
            epic_filter
                .map(FilterKind::Epic)
                .unwrap_or(FilterKind::Noop),
        )
}

trait IteratorExt<'a>: Iterator<Item = &'a ResponseNode> {
    fn filter_nodes(self, filter: FilterKind<'a>) -> NodeFilter<'a, Self>
    where
        Self: Sized,
    {
        NodeFilter::new(self, filter)
    }
}

impl<'a, I: Iterator<Item = &'a ResponseNode>> IteratorExt<'a> for I {}

struct NodeFilter<'a, I: Iterator> {
    it: I,
    typ: FilterKind<'a>,
}

impl<'a, I: Iterator<Item = &'a ResponseNode>> NodeFilter<'a, I> {
    const fn new(it: I, filter: FilterKind<'a>) -> Self {
        NodeFilter { it, typ: filter }
    }
}

impl<'a, I: Iterator<Item = &'a ResponseNode>> Iterator for NodeFilter<'_, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.it.next()?;
        match self.typ {
            FilterKind::Time(TimeFilter::WithinInclusive(before_date, after_date)) => {
                (item.datetime() >= after_date && item.datetime() <= before_date).then_some(item)
            }
            FilterKind::Time(TimeFilter::AfterInclusive(date)) => {
                (item.datetime() >= date).then_some(item)
            }
            FilterKind::Time(TimeFilter::BeforeInclusive(date)) => {
                (item.datetime() <= date).then_some(item)
            }
            FilterKind::Time(TimeFilter::Week { year, week }) => ({
                let isoweek = item.datetime().iso_week();
                isoweek.year() == year && isoweek.week() == week
            })
            .then_some(item),
            FilterKind::Group(GroupFilter::HasGroup(group)) => {
                item.has_group(group).then_some(item)
            }
            FilterKind::Epic(EpicFilter::HasEpic(epic)) => item.has_epic(epic).then_some(item),
            FilterKind::Group(GroupFilter::HasNoGroup) => {
                item.group_name().is_none().then_some(item)
            }
            FilterKind::Epic(EpicFilter::HasNoEpic) => item.epic_name().is_none().then_some(item),
            FilterKind::Noop => Some(item),
        }
    }
}

enum FilterKind<'a> {
    Time(TimeFilter),
    Group(GroupFilter<'a>),
    Epic(EpicFilter<'a>),
    Noop,
}

#[allow(unused)]
pub enum EpicFilter<'a> {
    HasEpic(&'a str),
    HasNoEpic,
}

#[allow(unused)]
pub enum GroupFilter<'a> {
    HasGroup(&'a str),
    HasNoGroup,
}

#[allow(unused)]
pub enum TimeFilter {
    AfterInclusive(NaiveDate),
    BeforeInclusive(NaiveDate),
    WithinInclusive(NaiveDate, NaiveDate),
    Week {
        year: i32,
        /// ISO week id.
        week: u32,
    },
}
