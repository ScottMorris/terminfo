// The `Tab` enum: the seven tabs of the TUI, their order, and navigation helpers.
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

/// The seven tabs of the application, in visual and number-key order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
    Overview,
    Colours,
    Attributes,
    Unicode,
    Input,
    Mouse,
    Graphics,
}

impl Tab {
    /// All tabs, in their canonical order.
    pub const ALL: [Tab; 7] = [
        Tab::Overview,
        Tab::Colours,
        Tab::Attributes,
        Tab::Unicode,
        Tab::Input,
        Tab::Mouse,
        Tab::Graphics,
    ];

    /// The title shown in the tab bar.
    pub fn title(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Colours => "Colours",
            Tab::Attributes => "Attributes",
            Tab::Unicode => "Unicode",
            Tab::Input => "Input",
            Tab::Mouse => "Mouse",
            Tab::Graphics => "Graphics",
        }
    }

    /// The zero-based position of this tab within `Tab::ALL`.
    pub fn index(self) -> usize {
        match self {
            Tab::Overview => 0,
            Tab::Colours => 1,
            Tab::Attributes => 2,
            Tab::Unicode => 3,
            Tab::Input => 4,
            Tab::Mouse => 5,
            Tab::Graphics => 6,
        }
    }

    /// The next tab, wrapping from the last back to the first.
    pub fn next(self) -> Tab {
        Tab::ALL[(self.index() + 1) % Tab::ALL.len()]
    }

    /// The previous tab, wrapping from the first back to the last.
    pub fn prev(self) -> Tab {
        Tab::ALL[(self.index() + Tab::ALL.len() - 1) % Tab::ALL.len()]
    }

    /// Maps a digit character `'1'..='7'` to the corresponding tab, one-based.
    pub fn from_digit(c: char) -> Option<Tab> {
        match c {
            '1' => Some(Tab::Overview),
            '2' => Some(Tab::Colours),
            '3' => Some(Tab::Attributes),
            '4' => Some(Tab::Unicode),
            '5' => Some(Tab::Input),
            '6' => Some(Tab::Mouse),
            '7' => Some(Tab::Graphics),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_has_seven_tabs() {
        assert_eq!(Tab::ALL.len(), 7);
    }

    #[test]
    fn next_wraps_forward_from_every_tab() {
        for &tab in Tab::ALL.iter() {
            let expected_index = (tab.index() + 1) % Tab::ALL.len();
            assert_eq!(tab.next().index(), expected_index);
        }
        assert_eq!(Tab::Graphics.next(), Tab::Overview);
    }

    #[test]
    fn prev_wraps_backward_from_every_tab() {
        for &tab in Tab::ALL.iter() {
            let expected_index = (tab.index() + Tab::ALL.len() - 1) % Tab::ALL.len();
            assert_eq!(tab.prev().index(), expected_index);
        }
        assert_eq!(Tab::Overview.prev(), Tab::Graphics);
    }

    #[test]
    fn next_then_prev_is_identity() {
        for &tab in Tab::ALL.iter() {
            assert_eq!(tab.next().prev(), tab);
        }
    }

    #[test]
    fn from_digit_covers_all_valid_digits() {
        assert_eq!(Tab::from_digit('1'), Some(Tab::Overview));
        assert_eq!(Tab::from_digit('2'), Some(Tab::Colours));
        assert_eq!(Tab::from_digit('3'), Some(Tab::Attributes));
        assert_eq!(Tab::from_digit('4'), Some(Tab::Unicode));
        assert_eq!(Tab::from_digit('5'), Some(Tab::Input));
        assert_eq!(Tab::from_digit('6'), Some(Tab::Mouse));
        assert_eq!(Tab::from_digit('7'), Some(Tab::Graphics));
    }

    #[test]
    fn from_digit_rejects_invalid_input() {
        assert_eq!(Tab::from_digit('0'), None);
        assert_eq!(Tab::from_digit('8'), None);
        assert_eq!(Tab::from_digit('a'), None);
        assert_eq!(Tab::from_digit(' '), None);
    }

    #[test]
    fn index_matches_all_position() {
        for (i, &tab) in Tab::ALL.iter().enumerate() {
            assert_eq!(tab.index(), i);
        }
    }
}
