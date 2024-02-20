//! Elements related to the `Block` base widget.
//!
//! This holds everything needed to display and configure a [`Block`].
//!
//! In its simplest form, a `Block` is a [border](Borders) around another widget. It can have a
//! [title](Block::title) and [padding](Block::padding).

use itertools::Itertools;
use strum::{Display, EnumString};

use crate::{
    buffer::Cell,
    prelude::*,
    symbols::border::{self, LineParts},
    widgets::Borders,
};

mod padding;
pub mod title;

pub use padding::Padding;
pub use title::{Position, Title};

/// Base widget to be used to display a box border around all [upper level ones](crate::widgets).
///
/// The borders can be configured with [`Block::borders`] and others. A block can have multiple
/// [`Title`] using [`Block::title`]. It can also be [styled](Block::style) and
/// [padded](Block::padding).
///
/// You can call the title methods multiple times to add multiple titles. Each title will be
/// rendered with a single space separating titles that are in the same position or alignment. When
/// both centered and non-centered titles are rendered, the centered space is calculated based on
/// the full width of the block, rather than the leftover width.
///
/// Titles are not rendered in the corners of the block unless there is no border on that edge.  
/// If the block is too small and multiple titles overlap, the border may get cut off at a corner.
///
/// ```plain
/// ┌With at least a left border───
///
/// Without left border───
/// ```
///
/// # Examples
///
/// ```
/// use ratatui::{prelude::*, widgets::*};
///
/// Block::default()
///     .title("Block")
///     .borders(Borders::LEFT | Borders::RIGHT)
///     .border_style(Style::default().fg(Color::White))
///     .border_type(BorderType::Rounded)
///     .style(Style::default().bg(Color::Black));
/// ```
///
/// You may also use multiple titles like in the following:
/// ```
/// use ratatui::{
///     prelude::*,
///     widgets::{block::*, *},
/// };
///
/// Block::default()
///     .title("Title 1")
///     .title(Title::from("Title 2").position(Position::Bottom));
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Block<'a> {
    /// List of titles
    titles: Vec<Title<'a>>,
    /// The style to be patched to all titles of the block
    titles_style: Style,
    /// The default alignment of the titles that don't have one
    titles_alignment: Alignment,
    /// The default position of the titles that don't have one
    titles_position: Position,
    /// Visible borders
    borders: Borders,
    /// Borders to merge with neighboring blocks
    merge_borders: Borders,
    /// Border style
    border_style: Style,
    /// The symbols used to render the border. The default is plain lines but one can choose to
    /// have rounded or doubled lines instead or a custom set of symbols
    border_set: border::Set,
    /// Widget style
    style: Style,
    /// Block padding
    padding: Padding,
}

/// The type of border of a [`Block`].
///
/// See the [`borders`](Block::borders) method of `Block` to configure its borders.
#[derive(Debug, Default, Display, EnumString, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BorderType {
    /// A plain, simple border.
    ///
    /// This is the default
    ///
    /// # Example
    ///
    /// ```plain
    /// ┌───────┐
    /// │       │
    /// └───────┘
    /// ```
    #[default]
    Plain,
    /// A plain border with rounded corners.
    ///
    /// # Example
    ///
    /// ```plain
    /// ╭───────╮
    /// │       │
    /// ╰───────╯
    /// ```
    Rounded,
    /// A doubled border.
    ///
    /// Note this uses one character that draws two lines.
    ///
    /// # Example
    ///
    /// ```plain
    /// ╔═══════╗
    /// ║       ║
    /// ╚═══════╝
    /// ```
    Double,
    /// A thick border.
    ///
    /// # Example
    ///
    /// ```plain
    /// ┏━━━━━━━┓
    /// ┃       ┃
    /// ┗━━━━━━━┛
    /// ```
    Thick,
    /// A border with a single line on the inside of a half block.
    ///
    /// # Example
    ///
    /// ```plain
    /// ▗▄▄▄▄▄▄▄▖
    /// ▐       ▌
    /// ▐       ▌
    /// ▝▀▀▀▀▀▀▀▘
    QuadrantInside,

    /// A border with a single line on the outside of a half block.
    ///
    /// # Example
    ///
    /// ```plain
    /// ▛▀▀▀▀▀▀▀▜
    /// ▌       ▐
    /// ▌       ▐
    /// ▙▄▄▄▄▄▄▄▟
    QuadrantOutside,
}

impl<'a> Block<'a> {
    /// Creates a new block with no [`Borders`] or [`Padding`].
    pub const fn new() -> Self {
        Self {
            titles: Vec::new(),
            titles_style: Style::new(),
            titles_alignment: Alignment::Left,
            titles_position: Position::Top,
            borders: Borders::NONE,
            merge_borders: Borders::NONE,
            border_style: Style::new(),
            border_set: BorderType::Plain.to_border_set(),
            style: Style::new(),
            padding: Padding::zero(),
        }
    }

    /// Create a new block with [all borders](Borders::ALL) shown
    pub const fn bordered() -> Self {
        let mut block = Block::new();
        block.borders = Borders::ALL;
        block
    }

    /// Adds a title to the block.
    ///
    /// The `title` function allows you to add a title to the block. You can call this function
    /// multiple times to add multiple titles.
    ///
    /// Each title will be rendered with a single space separating titles that are in the same
    /// position or alignment. When both centered and non-centered titles are rendered, the centered
    /// space is calculated based on the full width of the block, rather than the leftover width.
    ///
    /// You can provide any type that can be converted into [`Title`] including: strings, string
    /// slices (`&str`), borrowed strings (`Cow<str>`), [spans](crate::text::Span), or vectors of
    /// [spans](crate::text::Span) (`Vec<Span>`).
    ///
    /// By default, the titles will avoid being rendered in the corners of the block but will align
    /// against the left or right edge of the block if there is no border on that edge.  
    /// The following demonstrates this behavior, notice the second title is one character off to
    /// the left.
    ///
    /// ```plain
    /// ┌With at least a left border───
    ///
    /// Without left border───
    /// ```
    ///
    /// Note: If the block is too small and multiple titles overlap, the border might get cut off at
    /// a corner.
    ///
    /// # Example
    ///
    /// The following example demonstrates:
    /// - Default title alignment
    /// - Multiple titles (notice "Center" is centered according to the full with of the block, not
    /// the leftover space)
    /// - Two titles with the same alignment (notice the left titles are separated)
    /// ```
    /// use ratatui::{
    ///     prelude::*,
    ///     widgets::{block::*, *},
    /// };
    ///
    /// Block::default()
    ///     .title("Title") // By default in the top left corner
    ///     .title(Title::from("Left").alignment(Alignment::Left)) // also on the left
    ///     .title(Title::from("Right").alignment(Alignment::Right))
    ///     .title(Title::from("Center").alignment(Alignment::Center));
    /// // Renders
    /// // ┌Title─Left────Center─────────Right┐
    /// ```
    ///
    /// # See also
    ///
    /// Titles attached to a block can have default behaviors. See
    /// - [`Block::title_style`]
    /// - [`Block::title_alignment`]
    /// - [`Block::title_position`]
    pub fn title<T>(mut self, title: T) -> Block<'a>
    where
        T: Into<Title<'a>>,
    {
        self.titles.push(title.into());
        self
    }

    /// Adds a title to the top of the block.
    ///
    /// You can provide any type that can be converted into [`Line`] including: strings, string
    /// slices (`&str`), borrowed strings (`Cow<str>`), [spans](crate::text::Span), or vectors of
    /// [spans](crate::text::Span) (`Vec<Span>`).
    ///
    /// # Example
    ///
    /// ```
    /// # use ratatui::{ prelude::*, widgets::* };
    /// Block::bordered()
    ///     .title_top("Left1") // By default in the top left corner
    ///     .title_top(Line::from("Left2").left_aligned())
    ///     .title_top(Line::from("Right").right_aligned())
    ///     .title_top(Line::from("Center").centered());
    ///
    /// // Renders
    /// // ┌Left1─Left2───Center─────────Right┐
    /// // │                                  │
    /// // └──────────────────────────────────┘
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title_top<T: Into<Line<'a>>>(mut self, title: T) -> Self {
        let title = Title::from(title).position(Position::Top);
        self.titles.push(title);
        self
    }

    /// Adds a title to the bottom of the block.
    ///
    /// You can provide any type that can be converted into [`Line`] including: strings, string
    /// slices (`&str`), borrowed strings (`Cow<str>`), [spans](crate::text::Span), or vectors of
    /// [spans](crate::text::Span) (`Vec<Span>`).
    ///
    /// # Example
    ///
    /// ```
    /// # use ratatui::{ prelude::*, widgets::* };
    /// Block::bordered()
    ///     .title_bottom("Left1") // By default in the top left corner
    ///     .title_bottom(Line::from("Left2").left_aligned())
    ///     .title_bottom(Line::from("Right").right_aligned())
    ///     .title_bottom(Line::from("Center").centered());
    ///
    /// // Renders
    /// // ┌──────────────────────────────────┐
    /// // │                                  │
    /// // └Left1─Left2───Center─────────Right┘
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title_bottom<T: Into<Line<'a>>>(mut self, title: T) -> Self {
        let title = Title::from(title).position(Position::Bottom);
        self.titles.push(title);
        self
    }

    /// Applies the style to all titles.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// If a [`Title`] already has a style, the title's style will add on top of this one.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title_style<S: Into<Style>>(mut self, style: S) -> Block<'a> {
        self.titles_style = style.into();
        self
    }

    /// Sets the default [`Alignment`] for all block titles.
    ///
    /// Titles that explicitly set an [`Alignment`] will ignore this.
    ///
    /// # Example
    ///
    /// This example aligns all titles in the center except the "right" title which explicitly sets
    /// [`Alignment::Right`].
    /// ```
    /// use ratatui::{
    ///     prelude::*,
    ///     widgets::{block::*, *},
    /// };
    ///
    /// Block::default()
    ///     // This title won't be aligned in the center
    ///     .title(Title::from("right").alignment(Alignment::Right))
    ///     .title("foo")
    ///     .title("bar")
    ///     .title_alignment(Alignment::Center);
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn title_alignment(mut self, alignment: Alignment) -> Block<'a> {
        self.titles_alignment = alignment;
        self
    }

    /// Sets the default [`Position`] for all block [titles](Title).
    ///
    /// Titles that explicitly set a [`Position`] will ignore this.
    ///
    /// # Example
    ///
    /// This example positions all titles on the bottom except the "top" title which explicitly sets
    /// [`Position::Top`].
    /// ```
    /// use ratatui::{
    ///     prelude::*,
    ///     widgets::{block::*, *},
    /// };
    ///
    /// Block::default()
    ///     // This title won't be aligned in the center
    ///     .title(Title::from("top").position(Position::Top))
    ///     .title("foo")
    ///     .title("bar")
    ///     .title_position(Position::Bottom);
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn title_position(mut self, position: Position) -> Block<'a> {
        self.titles_position = position;
        self
    }

    /// Defines the style of the borders.
    ///
    /// If a [`Block::style`] is defined, `border_style` will be applied on top of it.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// # Example
    ///
    /// This example shows a `Block` with blue borders.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .borders(Borders::ALL)
    ///     .border_style(Style::new().blue());
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn border_style<S: Into<Style>>(mut self, style: S) -> Block<'a> {
        self.border_style = style.into();
        self
    }

    /// Defines the block style.
    ///
    /// This is the most generic [`Style`] a block can receive, it will be merged with any other
    /// more specific style. Elements can be styled further with [`Block::title_style`] and
    /// [`Block::border_style`].
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This will also apply to the widget inside that block, unless the inner widget is styled.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Block<'a> {
        self.style = style.into();
        self
    }

    /// Defines which borders to display.
    ///
    /// [`Borders`] can also be styled with [`Block::border_style`] and [`Block::border_type`].
    ///
    /// # Examples
    ///
    /// Simply show all borders.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default().borders(Borders::ALL);
    /// ```
    ///
    /// Display left and right borders.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default().borders(Borders::LEFT | Borders::RIGHT);
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn borders(mut self, flag: Borders) -> Block<'a> {
        self.borders = flag;
        self
    }

    /// Sets the symbols used to display the border (e.g. single line, double line, thick or
    /// rounded borders).
    ///
    /// Setting this overwrites any custom [`border_set`](Block::border_set) that was set.
    ///
    /// See [`BorderType`] for the full list of available symbols.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .title("Block")
    ///     .borders(Borders::ALL)
    ///     .border_type(BorderType::Rounded);
    /// // Renders
    /// // ╭Block╮
    /// // │     │
    /// // ╰─────╯
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn border_type(mut self, border_type: BorderType) -> Block<'a> {
        self.border_set = border_type.to_border_set();
        self
    }

    /// Sets the symbols used to display the border as a [`crate::symbols::border::Set`].
    ///
    /// Setting this overwrites any [`border_type`](Block::border_type) that was set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default().title("Block").borders(Borders::ALL).border_set(symbols::border::DOUBLE);
    /// // Renders
    /// // ╔Block╗
    /// // ║     ║
    /// // ╚═════╝
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn border_set(mut self, border_set: border::Set) -> Block<'a> {
        self.border_set = border_set;
        self
    }

    /// Sets which borders will be merged with those of neighboring [`Block`]s.
    ///
    /// This will only work correctly if the neighboring block has the same border set. Merging
    /// borders [`BorderType::QuadrantInside`] or [`BorderType::QuadrantOutside`] may produce
    /// undesired results due to the merging algorithm being unable to detect the correct
    /// junction symbol.
    ///
    /// # Examples
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default().title("Block 1").borders(Borders::ALL);
    /// Block::default().title("Block 2").borders(Borders::ALL).merge_with(Borders::LEFT);
    /// // Renders
    /// // ┌Block─1┬Block─2┐
    /// // │       │       │
    /// // └───────┴───────┘
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn merge_with(mut self, borders: Borders) -> Block<'a> {
        self.merge_borders = borders;
        self
    }

    /// Compute the inner area of a block based on its border visibility rules.
    ///
    /// # Examples
    ///
    /// Draw a block nested within another block
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// # fn render_nested_block(frame: &mut Frame) {
    /// let outer_block = Block::default().title("Outer").borders(Borders::ALL);
    /// let inner_block = Block::default().title("Inner").borders(Borders::ALL);
    ///
    /// let outer_area = frame.size();
    /// let inner_area = outer_block.inner(outer_area);
    ///
    /// frame.render_widget(outer_block, outer_area);
    /// frame.render_widget(inner_block, inner_area);
    /// # }
    /// // Renders
    /// // ┌Outer────────┐
    /// // │┌Inner──────┐│
    /// // ││           ││
    /// // │└───────────┘│
    /// // └─────────────┘
    /// ```
    pub fn inner(&self, area: Rect) -> Rect {
        let mut inner = area;
        if self.borders.intersects(Borders::LEFT) && !self.merge_borders.intersects(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1).min(inner.right());
            inner.width = inner.width.saturating_sub(1);
        }
        if (self.borders.intersects(Borders::TOP) || self.have_title_at_position(Position::Top))
            && !self.merge_borders.intersects(Borders::TOP)
        {
            inner.y = inner.y.saturating_add(1).min(inner.bottom());
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.intersects(Borders::RIGHT) && !self.merge_borders.intersects(Borders::RIGHT)
        {
            inner.width = inner.width.saturating_sub(1);
        }
        if (self.borders.intersects(Borders::BOTTOM)
            || self.have_title_at_position(Position::Bottom))
            && !self.merge_borders.intersects(Borders::BOTTOM)
        {
            inner.height = inner.height.saturating_sub(1);
        }

        inner.x = inner.x.saturating_add(self.padding.left);
        inner.y = inner.y.saturating_add(self.padding.top);

        inner.width = inner
            .width
            .saturating_sub(self.padding.left + self.padding.right);
        inner.height = inner
            .height
            .saturating_sub(self.padding.top + self.padding.bottom);

        inner
    }

    fn have_title_at_position(&self, position: Position) -> bool {
        self.titles
            .iter()
            .any(|title| title.position.unwrap_or(self.titles_position) == position)
    }

    /// Defines the padding inside a `Block`.
    ///
    /// See [`Padding`] for more information.
    ///
    /// # Examples
    ///
    /// This renders a `Block` with no padding (the default).
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .borders(Borders::ALL)
    ///     .padding(Padding::zero());
    /// // Renders
    /// // ┌───────┐
    /// // │content│
    /// // └───────┘
    /// ```
    ///
    /// This example shows a `Block` with padding left and right ([`Padding::horizontal`]).
    /// Notice the two spaces before and after the content.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .borders(Borders::ALL)
    ///     .padding(Padding::horizontal(2));
    /// // Renders
    /// // ┌───────────┐
    /// // │  content  │
    /// // └───────────┘
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn padding(mut self, padding: Padding) -> Block<'a> {
        self.padding = padding;
        self
    }
}

impl BorderType {
    /// Convert this `BorderType` into the corresponding [`Set`](border::Set) of border symbols.
    pub const fn border_symbols(border_type: BorderType) -> border::Set {
        match border_type {
            BorderType::Plain => border::PLAIN,
            BorderType::Rounded => border::ROUNDED,
            BorderType::Double => border::DOUBLE,
            BorderType::Thick => border::THICK,
            BorderType::QuadrantInside => border::QUADRANT_INSIDE,
            BorderType::QuadrantOutside => border::QUADRANT_OUTSIDE,
        }
    }

    /// Convert this `BorderType` into the corresponding [`Set`](border::Set) of border symbols.
    pub const fn to_border_set(self) -> border::Set {
        Self::border_symbols(self)
    }
}

impl Widget for Block<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for Block<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area = self.compensate_area_rect(area).intersection(buf.area);
        if area.is_empty() {
            return;
        }
        buf.set_style(area, self.style);
        self.render_borders(area, buf);
        self.render_titles(area, buf);
    }
}

impl Block<'_> {
    fn render_borders(&self, area: Rect, buf: &mut Buffer) {
        // Adjust the border rects to avoid drawing over corners if merging. This allows
        // scrollbars to render correctly.
        let horizontal_border_rect = self.calculate_horizontal_border_rect(area);
        let vertical_border_rect = self.calculate_vertical_border_rect(area);

        self.render_left_side(vertical_border_rect, buf);
        self.render_top_side(horizontal_border_rect, buf);
        self.render_right_side(vertical_border_rect, buf);
        self.render_bottom_side(horizontal_border_rect, buf);

        self.render_bottom_right_corner(buf, area);
        self.render_top_right_corner(buf, area);
        self.render_bottom_left_corner(buf, area);
        self.render_top_left_corner(buf, area);
    }

    fn render_titles(&self, area: Rect, buf: &mut Buffer) {
        self.render_title_position(Position::Top, area, buf);
        self.render_title_position(Position::Bottom, area, buf);
    }

    fn render_title_position(&self, position: Position, area: Rect, buf: &mut Buffer) {
        // NOTE: the order in which these functions are called defines the overlapping behavior
        self.render_right_titles(position, area, buf);
        self.render_center_titles(position, area, buf);
        self.render_left_titles(position, area, buf);
    }

    /// Compensate for merging borders in the rect.
    ///
    /// This will grow the rect towards any merged borders to reclaim the space gained
    /// from not having to draw those borders.
    fn compensate_area_rect(&self, area: Rect) -> Rect {
        let mut rect = area;
        if self.merge_borders.intersects(Borders::LEFT) {
            rect.x = rect.x.saturating_sub(1);
            rect.width += 1;
        }
        if self.merge_borders.intersects(Borders::RIGHT) {
            rect.width += 1;
        }
        if self.merge_borders.intersects(Borders::TOP) {
            rect.y = rect.y.saturating_sub(1);
            rect.height += 1;
        }
        if self.merge_borders.intersects(Borders::BOTTOM) {
            rect.height += 1;
        }
        rect
    }

    /// Compensate for vertical merging borders in the rect for border drawing.
    ///
    /// This should be done to the rect used to draw the left and right borders to ensure that
    /// the existing corner will not be rendered over and can be properly merged.
    fn calculate_vertical_border_rect(&self, area: Rect) -> Rect {
        let mut rect = area;
        if self.merge_borders.intersects(Borders::TOP) {
            rect.y += 1;
            rect.height -= 1;
        }
        if self.merge_borders.intersects(Borders::BOTTOM) {
            rect.height -= 1;
        }
        rect
    }

    /// Compensate for horizontal merging borders in the rect for border drawing.
    ///
    /// This should be done to the rect used to draw the top and bottom borders to ensure that
    /// the existing corner will not be rendered over and can be properly merged.
    fn calculate_horizontal_border_rect(&self, area: Rect) -> Rect {
        let mut rect = area;
        if self.merge_borders.intersects(Borders::LEFT) {
            rect.x += 1;
            rect.width -= 1;
        }
        if self.merge_borders.intersects(Borders::RIGHT) {
            rect.width -= 1;
        }
        rect
    }

    fn render_left_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::LEFT) && !self.merge_borders.contains(Borders::LEFT) {
            for y in area.top()..area.bottom() {
                buf.get_mut(area.left(), y)
                    .set_symbol(self.border_set.vertical_left)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_top_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::TOP) && !self.merge_borders.contains(Borders::TOP) {
            for x in area.left()..area.right() {
                buf.get_mut(x, area.top())
                    .set_symbol(self.border_set.horizontal_top)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_right_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::RIGHT) && !self.merge_borders.contains(Borders::RIGHT) {
            let x = area.right() - 1;
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(self.border_set.vertical_right)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_bottom_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::BOTTOM) && !self.merge_borders.contains(Borders::BOTTOM) {
            let y = area.bottom() - 1;
            for x in area.left()..area.right() {
                buf.get_mut(x, y)
                    .set_symbol(self.border_set.horizontal_bottom)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_merged_corner(&self, cell: &mut Cell, borders_to_merge: Borders) -> bool {
        if borders_to_merge.is_empty() {
            return false;
        }

        let current_parts = self.border_set.line_parts_from_symbol(cell.symbol());
        if current_parts.is_none() {
            return false;
        }

        let target_parts = current_parts.unwrap() | LineParts::from(borders_to_merge);
        let corner_symbol = self.border_set.symbol_from_line_parts(target_parts);
        cell.set_symbol(corner_symbol).set_style(self.border_style);
        true
    }

    fn render_bottom_right_corner(&self, buf: &mut Buffer, area: Rect) {
        let corner_cell = buf.get_mut(area.right() - 1, area.bottom() - 1);
        let borders_to_merge = self
            .merge_borders
            .intersection(Borders::RIGHT | Borders::BOTTOM);

        if self.render_merged_corner(corner_cell, borders_to_merge) {
            return;
        }

        if self.borders.contains(Borders::RIGHT | Borders::BOTTOM) {
            corner_cell
                .set_symbol(self.border_set.bottom_right)
                .set_style(self.border_style);
        }
    }

    fn render_top_right_corner(&self, buf: &mut Buffer, area: Rect) {
        let corner_cell = buf.get_mut(area.right() - 1, area.top());
        let borders_to_merge = self
            .merge_borders
            .intersection(Borders::RIGHT | Borders::TOP);

        if self.render_merged_corner(corner_cell, borders_to_merge) {
            return;
        }

        if self.borders.contains(Borders::RIGHT | Borders::TOP) {
            corner_cell
                .set_symbol(self.border_set.top_right)
                .set_style(self.border_style);
        }
    }

    fn render_bottom_left_corner(&self, buf: &mut Buffer, area: Rect) {
        let corner_cell = buf.get_mut(area.left(), area.bottom() - 1);
        let borders_to_merge = self
            .merge_borders
            .intersection(Borders::LEFT | Borders::BOTTOM);

        if self.render_merged_corner(corner_cell, borders_to_merge) {
            return;
        }

        if self.borders.contains(Borders::LEFT | Borders::BOTTOM) {
            corner_cell
                .set_symbol(self.border_set.bottom_left)
                .set_style(self.border_style);
        }
    }

    fn render_top_left_corner(&self, buf: &mut Buffer, area: Rect) {
        let corner_cell = buf.get_mut(area.left(), area.top());
        let borders_to_merge = self
            .merge_borders
            .intersection(Borders::LEFT | Borders::TOP);

        if self.render_merged_corner(corner_cell, borders_to_merge) {
            return;
        }

        if self.borders.contains(Borders::LEFT | Borders::TOP) {
            corner_cell
                .set_symbol(self.border_set.top_left)
                .set_style(self.border_style);
        }
    }

    /// Render titles aligned to the right of the block
    ///
    /// Currently (due to the way lines are truncated), the right side of the leftmost title will
    /// be cut off if the block is too small to fit all titles. This is not ideal and should be
    /// the left side of that leftmost that is cut off. This is due to the line being truncated
    /// incorrectly. See https://github.com/ratatui-org/ratatui/issues/932
    fn render_right_titles(&self, position: Position, area: Rect, buf: &mut Buffer) {
        let titles = self.filtered_titles(position, Alignment::Right);
        let mut titles_area = self.titles_area(area, position);

        // render titles in reverse order to align them to the right
        for title in titles.rev() {
            if titles_area.is_empty() {
                break;
            }
            let title_width = title.content.width() as u16;
            let title_area = Rect {
                x: titles_area
                    .right()
                    .saturating_sub(title_width)
                    .max(titles_area.left()),
                width: title_width.min(titles_area.width),
                ..titles_area
            };
            buf.set_style(title_area, self.titles_style);
            title.content.render_ref(title_area, buf);

            // bump the width of the titles area to the left
            titles_area.width = titles_area
                .width
                .saturating_sub(title_width)
                .saturating_sub(1); // space between titles
        }
    }

    /// Render titles in the center of the block
    ///
    /// Currently this method aligns the titles to the left inside a centered area. This is not
    /// ideal and should be fixed in the future to align the titles to the center of the block and
    /// truncate both sides of the titles if the block is too small to fit all titles.
    fn render_center_titles(&self, position: Position, area: Rect, buf: &mut Buffer) {
        let titles = self
            .filtered_titles(position, Alignment::Center)
            .collect_vec();
        let total_width = titles
            .iter()
            .map(|title| title.content.width() as u16 + 1) // space between titles
            .sum::<u16>()
            .saturating_sub(1); // no space for the last title

        let titles_area = self.titles_area(area, position);
        let mut titles_area = Rect {
            x: titles_area.left() + (titles_area.width.saturating_sub(total_width) / 2),
            ..titles_area
        };
        for title in titles {
            if titles_area.is_empty() {
                break;
            }
            let title_width = title.content.width() as u16;
            let title_area = Rect {
                width: title_width.min(titles_area.width),
                ..titles_area
            };
            buf.set_style(title_area, self.titles_style);
            title.content.render_ref(title_area, buf);

            // bump the titles area to the right and reduce its width
            titles_area.x = titles_area.x.saturating_add(title_width + 1);
            titles_area.width = titles_area.width.saturating_sub(title_width + 1);
        }
    }

    /// Render titles aligned to the left of the block
    fn render_left_titles(&self, position: Position, area: Rect, buf: &mut Buffer) {
        let titles = self.filtered_titles(position, Alignment::Left);
        let mut titles_area = self.titles_area(area, position);
        for title in titles {
            if titles_area.is_empty() {
                break;
            }
            let title_width = title.content.width() as u16;
            let title_area = Rect {
                width: title_width.min(titles_area.width),
                ..titles_area
            };
            buf.set_style(title_area, self.titles_style);
            title.content.render_ref(title_area, buf);

            // bump the titles area to the right and reduce its width
            titles_area.x = titles_area.x.saturating_add(title_width + 1);
            titles_area.width = titles_area.width.saturating_sub(title_width + 1);
        }
    }

    /// An iterator over the titles that match the position and alignment
    fn filtered_titles(
        &self,
        position: Position,
        alignment: Alignment,
    ) -> impl DoubleEndedIterator<Item = &Title> {
        self.titles.iter().filter(move |title| {
            title.position.unwrap_or(self.titles_position) == position
                && title.alignment.unwrap_or(self.titles_alignment) == alignment
        })
    }

    /// An area that is one line tall and spans the width of the block excluding the borders and
    /// is positioned at the top or bottom of the block.
    fn titles_area(&self, area: Rect, position: Position) -> Rect {
        let left_border = u16::from(self.borders.contains(Borders::LEFT));
        let right_border = u16::from(self.borders.contains(Borders::RIGHT));
        Rect {
            x: area.left() + left_border,
            y: match position {
                Position::Top => area.top(),
                Position::Bottom => area.bottom() - 1,
            },
            width: area
                .width
                .saturating_sub(left_border)
                .saturating_sub(right_border),
            height: 1,
        }
    }
}

/// An extension trait for [`Block`] that provides some convenience methods.
///
/// This is implemented for [`Option<Block>`](Option) to simplify the common case of having a
/// widget with an optional block.
pub trait BlockExt {
    /// Return the inner area of the block if it is `Some`. Otherwise, returns `area`.
    ///
    /// This is a useful convenience method for widgets that have an `Option<Block>` field
    fn inner_if_some(&self, area: Rect) -> Rect;
}

impl BlockExt for Option<Block<'_>> {
    fn inner_if_some(&self, area: Rect) -> Rect {
        self.as_ref().map_or(area, |block| block.inner(area))
    }
}

impl<'a> Styled for Block<'a> {
    type Item = Block<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

#[cfg(test)]
mod tests {
    use strum::ParseError;

    use super::*;
    use crate::{
        assert_buffer_eq,
        layout::{Alignment, Rect},
        style::{Color, Modifier, Stylize},
    };

    #[test]
    fn create_with_all_borders() {
        let block = Block::bordered();
        assert_eq!(block.borders, Borders::all());
    }

    #[test]
    fn inner_takes_into_account_the_borders() {
        // No borders
        assert_eq!(
            Block::default().inner(Rect::default()),
            Rect::new(0, 0, 0, 0),
            "no borders, width=0, height=0"
        );
        assert_eq!(
            Block::default().inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 0, 1, 1),
            "no borders, width=1, height=1"
        );

        // Left border
        assert_eq!(
            Block::default()
                .borders(Borders::LEFT)
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 0, 0, 1),
            "left, width=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::LEFT)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(1, 0, 0, 1),
            "left, width=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::LEFT)
                .inner(Rect::new(0, 0, 2, 1)),
            Rect::new(1, 0, 1, 1),
            "left, width=2"
        );

        // Top border
        assert_eq!(
            Block::default()
                .borders(Borders::TOP)
                .inner(Rect::new(0, 0, 1, 0)),
            Rect::new(0, 0, 1, 0),
            "top, height=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::TOP)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 1, 1, 0),
            "top, height=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::TOP)
                .inner(Rect::new(0, 0, 1, 2)),
            Rect::new(0, 1, 1, 1),
            "top, height=2"
        );

        // Right border
        assert_eq!(
            Block::default()
                .borders(Borders::RIGHT)
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 0, 0, 1),
            "right, width=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::RIGHT)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 0, 0, 1),
            "right, width=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::RIGHT)
                .inner(Rect::new(0, 0, 2, 1)),
            Rect::new(0, 0, 1, 1),
            "right, width=2"
        );

        // Bottom border
        assert_eq!(
            Block::default()
                .borders(Borders::BOTTOM)
                .inner(Rect::new(0, 0, 1, 0)),
            Rect::new(0, 0, 1, 0),
            "bottom, height=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::BOTTOM)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 0, 1, 0),
            "bottom, height=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::BOTTOM)
                .inner(Rect::new(0, 0, 1, 2)),
            Rect::new(0, 0, 1, 1),
            "bottom, height=2"
        );

        // All borders
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::default()),
            Rect::new(0, 0, 0, 0),
            "all borders, width=0, height=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(1, 1, 0, 0),
            "all borders, width=1, height=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::new(0, 0, 2, 2)),
            Rect::new(1, 1, 0, 0),
            "all borders, width=2, height=2"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::new(0, 0, 3, 3)),
            Rect::new(1, 1, 1, 1),
            "all borders, width=3, height=3"
        );
    }

    #[test]
    fn inner_takes_into_account_the_title() {
        assert_eq!(
            Block::default().title("Test").inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 1, 0, 0),
        );
        assert_eq!(
            Block::default()
                .title(Title::from("Test").alignment(Alignment::Center))
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 1, 0, 0),
        );
        assert_eq!(
            Block::default()
                .title(Title::from("Test").alignment(Alignment::Right))
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 1, 0, 0),
        );
    }

    #[test]
    fn inner_takes_into_account_border_and_title() {
        let test_rect = Rect::new(0, 0, 0, 2);

        let top_top = Block::default()
            .title(Title::from("Test").position(Position::Top))
            .borders(Borders::TOP);
        assert_eq!(top_top.inner(test_rect), Rect::new(0, 1, 0, 1));

        let top_bot = Block::default()
            .title(Title::from("Test").position(Position::Top))
            .borders(Borders::BOTTOM);
        assert_eq!(top_bot.inner(test_rect), Rect::new(0, 1, 0, 0));

        let bot_top = Block::default()
            .title(Title::from("Test").position(Position::Bottom))
            .borders(Borders::TOP);
        assert_eq!(bot_top.inner(test_rect), Rect::new(0, 1, 0, 0));

        let bot_bot = Block::default()
            .title(Title::from("Test").position(Position::Bottom))
            .borders(Borders::BOTTOM);
        assert_eq!(bot_bot.inner(test_rect), Rect::new(0, 0, 0, 1));
    }

    #[test]
    fn inner_takes_into_account_merge() {
        let test_rect = Rect::new(1, 1, 5, 5);
        assert_eq!(
            Block::bordered().merge_with(Borders::LEFT).inner(test_rect),
            Rect::new(1, 2, 4, 3)
        );
        assert_eq!(
            Block::bordered().merge_with(Borders::TOP).inner(test_rect),
            Rect::new(2, 1, 3, 4)
        );
        assert_eq!(
            Block::bordered()
                .merge_with(Borders::TOP)
                .title("Test")
                .inner(test_rect),
            Rect::new(2, 1, 3, 4)
        );
        assert_eq!(
            Block::bordered()
                .merge_with(Borders::RIGHT)
                .inner(test_rect),
            Rect::new(2, 2, 4, 3)
        );
        assert_eq!(
            Block::bordered()
                .merge_with(Borders::BOTTOM)
                .inner(test_rect),
            Rect::new(2, 2, 3, 4)
        );
        assert_eq!(
            Block::bordered()
                .merge_with(Borders::BOTTOM)
                .title("Test")
                .inner(test_rect),
            Rect::new(2, 2, 3, 4)
        );
    }

    #[test]
    fn have_title_at_position_takes_into_account_all_positioning_declarations() {
        let block = Block::default();
        assert!(!block.have_title_at_position(Position::Top));
        assert!(!block.have_title_at_position(Position::Bottom));

        let block = Block::default().title(Title::from("Test").position(Position::Top));
        assert!(block.have_title_at_position(Position::Top));
        assert!(!block.have_title_at_position(Position::Bottom));

        let block = Block::default().title(Title::from("Test").position(Position::Bottom));
        assert!(!block.have_title_at_position(Position::Top));
        assert!(block.have_title_at_position(Position::Bottom));

        let block = Block::default()
            .title(Title::from("Test").position(Position::Top))
            .title_position(Position::Bottom);
        assert!(block.have_title_at_position(Position::Top));
        assert!(!block.have_title_at_position(Position::Bottom));

        let block = Block::default()
            .title(Title::from("Test").position(Position::Bottom))
            .title_position(Position::Top);
        assert!(!block.have_title_at_position(Position::Top));
        assert!(block.have_title_at_position(Position::Bottom));

        let block = Block::default()
            .title(Title::from("Test").position(Position::Top))
            .title(Title::from("Test").position(Position::Bottom));
        assert!(block.have_title_at_position(Position::Top));
        assert!(block.have_title_at_position(Position::Bottom));

        let block = Block::default()
            .title(Title::from("Test").position(Position::Top))
            .title(Title::from("Test"))
            .title_position(Position::Bottom);
        assert!(block.have_title_at_position(Position::Top));
        assert!(block.have_title_at_position(Position::Bottom));

        let block = Block::default()
            .title(Title::from("Test"))
            .title(Title::from("Test").position(Position::Bottom))
            .title_position(Position::Top);
        assert!(block.have_title_at_position(Position::Top));
        assert!(block.have_title_at_position(Position::Bottom));
    }

    #[test]
    fn border_type_can_be_const() {
        const _PLAIN: border::Set = BorderType::border_symbols(BorderType::Plain);
    }

    #[test]
    fn block_new() {
        assert_eq!(
            Block::new(),
            Block {
                titles: Vec::new(),
                titles_style: Style::new(),
                titles_alignment: Alignment::Left,
                titles_position: Position::Top,
                borders: Borders::NONE,
                merge_borders: Borders::NONE,
                border_style: Style::new(),
                border_set: BorderType::Plain.to_border_set(),
                style: Style::new(),
                padding: Padding::zero(),
            }
        )
    }

    #[test]
    fn block_can_be_const() {
        const _DEFAULT_STYLE: Style = Style::new();
        const _DEFAULT_PADDING: Padding = Padding::uniform(1);
        const _DEFAULT_BLOCK: Block = Block::new()
            // the following methods are no longer const because they use Into<Style>
            // .style(_DEFAULT_STYLE)           // no longer const
            // .border_style(_DEFAULT_STYLE)    // no longer const
            // .title_style(_DEFAULT_STYLE)     // no longer const
            .title_alignment(Alignment::Left)
            .title_position(Position::Top)
            .borders(Borders::ALL)
            .padding(_DEFAULT_PADDING);
    }

    /// This test ensures that we have some coverage on the Style::from() implementations
    #[test]
    fn block_style() {
        // nominal style
        let block = Block::default().style(Style::new().red());
        assert_eq!(block.style, Style::new().red());

        // auto-convert from Color
        let block = Block::default().style(Color::Red);
        assert_eq!(block.style, Style::new().red());

        // auto-convert from (Color, Color)
        let block = Block::default().style((Color::Red, Color::Blue));
        assert_eq!(block.style, Style::new().red().on_blue());

        // auto-convert from Modifier
        let block = Block::default().style(Modifier::BOLD | Modifier::ITALIC);
        assert_eq!(block.style, Style::new().bold().italic());

        // auto-convert from (Modifier, Modifier)
        let block = Block::default().style((Modifier::BOLD | Modifier::ITALIC, Modifier::DIM));
        assert_eq!(block.style, Style::new().bold().italic().not_dim());

        // auto-convert from (Color, Modifier)
        let block = Block::default().style((Color::Red, Modifier::BOLD));
        assert_eq!(block.style, Style::new().red().bold());

        // auto-convert from (Color, Color, Modifier)
        let block = Block::default().style((Color::Red, Color::Blue, Modifier::BOLD));
        assert_eq!(block.style, Style::new().red().on_blue().bold());

        // auto-convert from (Color, Color, Modifier, Modifier)
        let block = Block::default().style((
            Color::Red,
            Color::Blue,
            Modifier::BOLD | Modifier::ITALIC,
            Modifier::DIM,
        ));
        assert_eq!(
            block.style,
            Style::new().red().on_blue().bold().italic().not_dim()
        );
    }

    #[test]
    fn can_be_stylized() {
        let block = Block::default().black().on_white().bold().not_dim();
        assert_eq!(
            block.style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::DIM)
        )
    }

    #[test]
    fn title() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        use Alignment::*;
        use Position::*;
        Block::bordered()
            .title(Title::from("A").position(Top).alignment(Left))
            .title(Title::from("B").position(Top).alignment(Center))
            .title(Title::from("C").position(Top).alignment(Right))
            .title(Title::from("D").position(Bottom).alignment(Left))
            .title(Title::from("E").position(Bottom).alignment(Center))
            .title(Title::from("F").position(Bottom).alignment(Right))
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌A─────B─────C┐",
                "│             │",
                "└D─────E─────F┘",
            ])
        );
    }

    #[test]
    fn title_top_bottom() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::bordered()
            .title_top(Line::raw("A").left_aligned())
            .title_top(Line::raw("B").centered())
            .title_top(Line::raw("C").right_aligned())
            .title_bottom(Line::raw("D").left_aligned())
            .title_bottom(Line::raw("E").centered())
            .title_bottom(Line::raw("F").right_aligned())
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌A─────B─────C┐",
                "│             │",
                "└D─────E─────F┘",
            ])
        );
    }

    #[test]
    fn title_alignment() {
        let tests = vec![
            (Alignment::Left, "test    "),
            (Alignment::Center, "  test  "),
            (Alignment::Right, "    test"),
        ];
        for (alignment, expected) in tests {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 1));
            Block::default()
                .title("test")
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);
            assert_buffer_eq!(buffer, Buffer::with_lines(vec![expected]));
        }
    }

    #[test]
    fn title_alignment_overrides_block_title_alignment() {
        let tests = vec![
            (Alignment::Right, Alignment::Left, "test    "),
            (Alignment::Left, Alignment::Center, "  test  "),
            (Alignment::Center, Alignment::Right, "    test"),
        ];
        for (block_title_alignment, alignment, expected) in tests {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 1));
            Block::default()
                .title(Title::from("test").alignment(alignment))
                .title_alignment(block_title_alignment)
                .render(buffer.area, &mut buffer);
            assert_buffer_eq!(buffer, Buffer::with_lines(vec![expected]));
        }
    }

    /// This is a regression test for bug https://github.com/ratatui-org/ratatui/issues/929
    #[test]
    fn render_right_aligned_empty_title() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .title("")
            .title_alignment(Alignment::Right)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "               ",
                "               ",
                "               ",
            ])
        );
    }

    #[test]
    fn title_position() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 2));
        Block::default()
            .title("test")
            .title_position(Position::Bottom)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(buffer, Buffer::with_lines(vec!["    ", "test"]));
    }

    #[test]
    fn title_content_style() {
        for alignment in [Alignment::Left, Alignment::Center, Alignment::Right] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
            Block::default()
                .title("test".yellow())
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);

            let mut expected_buffer = Buffer::with_lines(vec!["test"]);
            expected_buffer.set_style(Rect::new(0, 0, 4, 1), Style::new().yellow());

            assert_buffer_eq!(buffer, expected_buffer);
        }
    }

    #[test]
    fn block_title_style() {
        for alignment in [Alignment::Left, Alignment::Center, Alignment::Right] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
            Block::default()
                .title("test")
                .title_style(Style::new().yellow())
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);

            let mut expected_buffer = Buffer::with_lines(vec!["test"]);
            expected_buffer.set_style(Rect::new(0, 0, 4, 1), Style::new().yellow());

            assert_buffer_eq!(buffer, expected_buffer);
        }
    }

    #[test]
    fn title_style_overrides_block_title_style() {
        for alignment in [Alignment::Left, Alignment::Center, Alignment::Right] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
            Block::default()
                .title("test".yellow())
                .title_style(Style::new().green().on_red())
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);

            let mut expected_buffer = Buffer::with_lines(vec!["test"]);
            expected_buffer.set_style(Rect::new(0, 0, 4, 1), Style::new().yellow().on_red());

            assert_buffer_eq!(buffer, expected_buffer);
        }
    }

    #[test]
    fn title_border_style() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .title("test")
            .borders(Borders::ALL)
            .border_style(Style::new().yellow())
            .render(buffer.area, &mut buffer);

        let mut expected_buffer = Buffer::with_lines(vec![
            "┌test─────────┐",
            "│             │",
            "└─────────────┘",
        ]);
        expected_buffer.set_style(Rect::new(0, 0, 15, 3), Style::new().yellow());
        expected_buffer.set_style(Rect::new(1, 1, 13, 1), Style::reset());

        assert_buffer_eq!(buffer, expected_buffer);
    }

    #[test]
    fn border_type_to_string() {
        assert_eq!(format!("{}", BorderType::Plain), "Plain");
        assert_eq!(format!("{}", BorderType::Rounded), "Rounded");
        assert_eq!(format!("{}", BorderType::Double), "Double");
        assert_eq!(format!("{}", BorderType::Thick), "Thick");
    }

    #[test]
    fn border_type_from_str() {
        assert_eq!("Plain".parse(), Ok(BorderType::Plain));
        assert_eq!("Rounded".parse(), Ok(BorderType::Rounded));
        assert_eq!("Double".parse(), Ok(BorderType::Double));
        assert_eq!("Thick".parse(), Ok(BorderType::Thick));
        assert_eq!("".parse::<BorderType>(), Err(ParseError::VariantNotFound));
    }

    #[test]
    fn render_plain_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌─────────────┐",
                "│             │",
                "└─────────────┘"
            ])
        );
    }

    #[test]
    fn render_rounded_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "╭─────────────╮",
                "│             │",
                "╰─────────────╯"
            ])
        );
    }

    #[test]
    fn render_double_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "╔═════════════╗",
                "║             ║",
                "╚═════════════╝"
            ])
        );
    }

    #[test]
    fn render_quadrant_inside() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantInside)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "▗▄▄▄▄▄▄▄▄▄▄▄▄▄▖",
                "▐             ▌",
                "▝▀▀▀▀▀▀▀▀▀▀▀▀▀▘",
            ])
        );
    }

    #[test]
    fn render_border_quadrant_outside() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantOutside)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "▛▀▀▀▀▀▀▀▀▀▀▀▀▀▜",
                "▌             ▐",
                "▙▄▄▄▄▄▄▄▄▄▄▄▄▄▟",
            ])
        );
    }

    #[test]
    fn render_solid_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┏━━━━━━━━━━━━━┓",
                "┃             ┃",
                "┗━━━━━━━━━━━━━┛"
            ])
        );
    }

    #[test]
    fn render_custom_border_set() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::Set {
                top_left: "1",
                top_right: "2",
                bottom_left: "3",
                bottom_right: "4",
                vertical_left: "L",
                vertical_right: "R",
                horizontal_top: "T",
                horizontal_bottom: "B",
                vertical_t_left: " ",
                vertical_t_right: " ",
                horizontal_t_down: " ",
                horizontal_t_up: " ",
                cross: " ",
            })
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "1TTTTTTTTTTTTT2",
                "L             R",
                "3BBBBBBBBBBBBB4",
            ])
        );
    }

    #[test]
    fn render_merged_plain_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 5);
        let block2_area = Rect::new(10, 0, 5, 3);
        let block3_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .merge_with(Borders::LEFT)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .merge_with(Borders::LEFT | Borders::TOP)
            .render(block3_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌────────┬────┐",
                "│        │    │",
                "│        ├────┤",
                "│        │    │",
                "└────────┴────┘"
            ])
        );
    }

    #[test]
    fn render_merged_rounded_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 5);
        let block2_area = Rect::new(10, 0, 5, 3);
        let block3_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .merge_with(Borders::LEFT)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .merge_with(Borders::TOP | Borders::LEFT)
            .render(block3_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "╭────────┬────╮",
                "│        │    │",
                "│        ├────┤",
                "│        │    │",
                "╰────────┴────╯"
            ])
        );
    }

    #[test]
    fn render_merged_solid_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 5);
        let block2_area = Rect::new(10, 0, 5, 3);
        let block3_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .merge_with(Borders::LEFT)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .merge_with(Borders::TOP | Borders::LEFT)
            .render(block3_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┏━━━━━━━━┳━━━━┓",
                "┃        ┃    ┃",
                "┃        ┣━━━━┫",
                "┃        ┃    ┃",
                "┗━━━━━━━━┻━━━━┛",
            ])
        );
    }

    #[test]
    fn render_merged_double_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 5);
        let block2_area = Rect::new(10, 0, 5, 3);
        let block3_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .merge_with(Borders::LEFT)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .merge_with(Borders::TOP | Borders::LEFT)
            .render(block3_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "╔════════╦════╗",
                "║        ║    ║",
                "║        ╠════╣",
                "║        ║    ║",
                "╚════════╩════╝",
            ])
        );
    }

    #[test]
    fn render_merged_quadrant_outside() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 5);
        let block2_area = Rect::new(10, 0, 5, 3);
        let block3_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantOutside)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantOutside)
            .merge_with(Borders::LEFT)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantOutside)
            .merge_with(Borders::TOP | Borders::LEFT)
            .render(block3_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "▛▀▀▀▀▀▀▀▀█▀▀▀▀▜",
                "▌        ▐    ▐",
                "▌        █▄▄▄▄█",
                "▌        ▐    ▐",
                "▙▄▄▄▄▄▄▄▄█▄▄▄▄▟",
            ])
        );
    }

    #[test]
    fn render_merged_quadrant_inside() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 5);
        let block2_area = Rect::new(10, 0, 5, 3);
        let block3_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantInside)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantInside)
            .merge_with(Borders::LEFT)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantInside)
            .merge_with(Borders::TOP | Borders::LEFT)
            .render(block3_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "▗▄▄▄▄▄▄▄▄█▄▄▄▄▖",
                "▐        ▌    ▌",
                "▐        █▀▀▀▀█",
                "▐        ▌    ▌",
                "▝▀▀▀▀▀▀▀▀█▀▀▀▀▘",
            ])
        );
    }

    #[test]
    fn render_merged_with_cross() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 3);
        let block2_area = Rect::new(0, 3, 10, 2);
        let block3_area = Rect::new(10, 0, 5, 3);
        let block4_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .merge_with(Borders::TOP)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .merge_with(Borders::LEFT)
            .render(block3_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .merge_with(Borders::LEFT | Borders::TOP)
            .render(block4_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌────────┬────┐",
                "│        │    │",
                "├────────┼────┤",
                "│        │    │",
                "└────────┴────┘"
            ])
        );
    }

    #[test]
    fn render_merged_custom_set() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 3);
        let block2_area = Rect::new(0, 3, 10, 2);
        let block3_area = Rect::new(10, 0, 5, 3);
        let block4_area = Rect::new(10, 3, 5, 2);
        let set = border::Set {
            top_left: "1",
            top_right: "2",
            bottom_left: "3",
            bottom_right: "4",
            vertical_left: "L",
            vertical_right: "R",
            horizontal_top: "T",
            horizontal_bottom: "B",
            vertical_t_left: "<",
            vertical_t_right: ">",
            horizontal_t_down: "v",
            horizontal_t_up: "^",
            cross: "C",
        };
        Block::default()
            .borders(Borders::ALL)
            .border_set(set)
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_set(set)
            .merge_with(Borders::TOP)
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_set(set)
            .merge_with(Borders::LEFT)
            .render(block3_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_set(set)
            .merge_with(Borders::LEFT | Borders::TOP)
            .render(block4_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "1TTTTTTTTvTTTT2",
                "L        R    R",
                ">BBBBBBBBCBBBB<",
                "L        R    R",
                "3BBBBBBBB^BBBB4"
            ])
        );
    }

    #[test]
    fn render_merged_with_title() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 5));
        let block1_area = Rect::new(0, 0, 10, 3);
        let block2_area = Rect::new(0, 3, 10, 2);
        let block3_area = Rect::new(10, 0, 5, 3);
        let block4_area = Rect::new(10, 3, 5, 2);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .title(
                Title::from("^1")
                    .position(Position::Bottom)
                    .alignment(Alignment::Right),
            )
            .render(block1_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .merge_with(Borders::TOP)
            .title("2")
            .render(block2_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .merge_with(Borders::LEFT)
            .title("3")
            .render(block3_area, &mut buffer);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .title("4")
            .merge_with(Borders::LEFT | Borders::TOP)
            .render(block4_area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌────────┬3───┐",
                "│        │    │",
                "├2─────^1┼4───┤",
                "│        │    │",
                "└────────┴────┘"
            ])
        );
    }
}
