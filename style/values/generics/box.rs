/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for box properties.

use crate::derives::*;
use crate::typed_om::{ToTyped, TypedValue};
use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::generics::Optional;
use crate::values::{ComputeSquaredDistance, SquaredDistance};
use crate::Zero;
use std::fmt::{self, Write};
use style_traits::values::SequenceWriter;
use style_traits::{CssWriter, ToCss};
use thin_vec::ThinVec;

#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    FromPrimitive,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum BaselineShiftKeyword {
    /// Lower by the offset appropriate for subscripts of the parent’s box. The UA may use the
    /// parent’s font metrics to find this offset; otherwise it defaults to dropping by one
    /// fifth of the parent’s used font-size.
    Sub,
    /// Raise by the offset appropriate for superscripts of the parent’s box. The UA may use the
    /// parent’s font metrics to find this offset; otherwise it defaults to raising by one third
    /// of the parent’s used font-size.
    Super,
    /// Align the line-over edge of the aligned subtree with the line-over edge of the line box.
    Top,
    /// Align the center of the aligned subtree with the center of the line box.
    Center,
    /// Align the line-under edge of the aligned subtree with the line-under edge of the line box.
    Bottom,
}

/// A generic value for the `baseline-shift` property.
/// https://drafts.csswg.org/css-inline-3/#baseline-shift
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericBaselineShift<LengthPercentage> {
    /// One of the baseline-shift keywords
    Keyword(BaselineShiftKeyword),
    /// Raise (positive value) or lower (negative value) by the specified length or specified percentage of the line-height.
    Length(LengthPercentage),
    /// The initial or shorthand-omitted zero shift. Its numeric value is zero,
    /// but it preserves the baseline category for callers that need CSS boundary intent.
    #[value_info(skip)]
    ImplicitZero,
}

pub use self::GenericBaselineShift as BaselineShift;

impl<L: Zero> BaselineShift<L> {
    /// Returns an explicit numeric `0` value.
    #[inline]
    pub fn zero() -> Self {
        BaselineShift::Length(Zero::zero())
    }
}

impl<L> BaselineShift<L> {
    /// Returns the initial or shorthand-omitted zero shift.
    #[inline]
    pub fn implicit_zero() -> Self {
        Self::ImplicitZero
    }
}

impl<L: ToCss + Zero> ToCss for BaselineShift<L> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self {
            Self::Keyword(keyword) => keyword.to_css(dest),
            Self::Length(length) => length.to_css(dest),
            Self::ImplicitZero => L::zero().to_css(dest),
        }
    }
}

impl<L: ToTyped + Zero> ToTyped for BaselineShift<L> {
    fn to_typed(&self, dest: &mut ThinVec<TypedValue>) -> Result<(), ()> {
        match self {
            Self::Keyword(keyword) => keyword.to_typed(dest),
            Self::Length(length) => length.to_typed(dest),
            Self::ImplicitZero => L::zero().to_typed(dest),
        }
    }
}

impl<L: Animate + Zero> Animate for BaselineShift<L> {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(match (self, other) {
            (Self::ImplicitZero, Self::ImplicitZero) => Self::ImplicitZero,
            (Self::ImplicitZero, Self::Length(other)) => {
                Self::Length(L::zero().animate(other, procedure)?)
            },
            (Self::Length(this), Self::ImplicitZero) => {
                Self::Length(this.animate(&L::zero(), procedure)?)
            },
            (Self::Length(this), Self::Length(other)) => {
                Self::Length(this.animate(other, procedure)?)
            },
            (Self::Keyword(this), Self::Keyword(other)) => {
                Self::Keyword(this.animate(other, procedure)?)
            },
            _ => return Err(()),
        })
    }
}

impl<L: ComputeSquaredDistance + Zero> ComputeSquaredDistance for BaselineShift<L> {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (Self::ImplicitZero, Self::ImplicitZero) => Ok(SquaredDistance::from_sqrt(0.)),
            (Self::ImplicitZero, Self::Length(other)) => L::zero().compute_squared_distance(other),
            (Self::Length(this), Self::ImplicitZero) => this.compute_squared_distance(&L::zero()),
            (Self::Length(this), Self::Length(other)) => this.compute_squared_distance(other),
            (Self::Keyword(this), Self::Keyword(other)) => this.compute_squared_distance(other),
            _ => Err(()),
        }
    }
}

impl<L> ToAnimatedZero for BaselineShift<L> {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

#[cfg(test)]
mod baseline_shift_tests {
    use super::*;
    use crate::typed_om::NumericValue;
    use crate::values::computed::{Length, LengthPercentage};

    type Shift = BaselineShift<LengthPercentage>;

    fn length(value: f32) -> Shift {
        Shift::Length(LengthPercentage::new_length(Length::new(value)))
    }

    #[test]
    fn implicit_zero_preserves_category_but_exposes_numeric_zero() {
        let implicit = Shift::implicit_zero();
        let explicit = Shift::zero();
        assert_ne!(implicit, explicit);
        assert_eq!(implicit.to_css_string().to_string(), "0px");
        assert_eq!(implicit.to_css_string(), explicit.to_css_string());

        for value in [implicit, explicit] {
            let mut typed = ThinVec::new();
            value.to_typed(&mut typed).unwrap();
            assert_eq!(typed.len(), 1);
            let TypedValue::Numeric(NumericValue::Unit(unit)) = &typed[0] else {
                panic!("zero shift must expose a numeric unit: {typed:?}");
            };
            assert_eq!(unit.value, 0.);
            assert_eq!(unit.unit_str(), "px");
        }

        let specified =
            BaselineShift::<crate::values::specified::LengthPercentage>::implicit_zero();
        assert_eq!(specified.to_css_string().to_string(), "0px");
        let mut typed = ThinVec::new();
        specified.to_typed(&mut typed).unwrap();
        assert!(
            matches!(typed.as_slice(), [TypedValue::Numeric(NumericValue::Unit(unit))]
            if unit.value == 0. && unit.unit_str() == "px")
        );

        let mut keywords = Vec::new();
        <BaselineShift<crate::values::specified::LengthPercentage> as style_traits::SpecifiedValueInfo>::collect_completion_keywords(&mut |values| {
            keywords.extend(values.iter().copied());
        });
        assert!(!keywords.contains(&"implicit-zero"));
    }

    #[test]
    fn implicit_zero_interpolates_as_numeric_zero_in_both_directions() {
        let implicit = Shift::implicit_zero();
        let explicit = length(8.);
        let quarter = Procedure::Interpolate { progress: 0.25 };
        assert_eq!(implicit.animate(&explicit, quarter).unwrap(), length(2.));
        assert_eq!(explicit.animate(&implicit, quarter).unwrap(), length(6.));
        assert_eq!(implicit.animate(&implicit, quarter).unwrap(), implicit);
        assert_eq!(
            implicit.animate(&Shift::zero(), quarter).unwrap(),
            Shift::zero()
        );
        assert_eq!(
            Shift::zero().animate(&implicit, quarter).unwrap(),
            Shift::zero()
        );
        assert_eq!(
            implicit.animate(&explicit, Procedure::Add).unwrap(),
            explicit
        );
        assert_eq!(
            explicit
                .animate(&implicit, Procedure::Accumulate { count: 2 })
                .unwrap(),
            length(16.)
        );

        let distance = SquaredDistance::from_sqrt(8.);
        assert_eq!(
            implicit.compute_squared_distance(&explicit).unwrap(),
            distance
        );
        assert_eq!(
            explicit.compute_squared_distance(&implicit).unwrap(),
            distance
        );
        assert_eq!(
            implicit.compute_squared_distance(&Shift::zero()).unwrap(),
            SquaredDistance::from_sqrt(0.)
        );
        assert_eq!(
            implicit.compute_squared_distance(&implicit).unwrap(),
            SquaredDistance::from_sqrt(0.)
        );
    }

    #[test]
    fn implicit_zero_does_not_change_keyword_animation_rules() {
        let implicit = Shift::implicit_zero();
        let numeric = Shift::zero();
        let sub = Shift::Keyword(BaselineShiftKeyword::Sub);
        let super_ = Shift::Keyword(BaselineShiftKeyword::Super);
        let half = Procedure::Interpolate { progress: 0.5 };
        assert_eq!(sub.animate(&sub, half).unwrap(), sub);
        assert_eq!(
            sub.compute_squared_distance(&sub).unwrap(),
            SquaredDistance::from_sqrt(0.)
        );
        for (this, other) in [
            (&implicit, &sub),
            (&sub, &implicit),
            (&numeric, &sub),
            (&sub, &numeric),
            (&sub, &super_),
        ] {
            assert!(this.animate(other, half).is_err());
            assert!(this.compute_squared_distance(other).is_err());
        }
        assert!(implicit.to_animated_zero().is_err());
    }
}

/// https://drafts.csswg.org/css-sizing-4/#intrinsic-size-override
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToAnimatedValue,
    ToAnimatedZero,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[value_info(other_values = "auto")]
#[repr(C, u8)]
pub enum GenericContainIntrinsicSize<L> {
    /// The keyword `none`.
    None,
    /// The keywords 'auto none',
    AutoNone,
    /// A non-negative length.
    Length(L),
    /// "auto <Length>"
    AutoLength(L),
}

pub use self::GenericContainIntrinsicSize as ContainIntrinsicSize;

impl<L: ToCss> ToCss for ContainIntrinsicSize<L> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            Self::None => dest.write_str("none"),
            Self::AutoNone => dest.write_str("auto none"),
            Self::Length(ref l) => l.to_css(dest),
            Self::AutoLength(ref l) => {
                dest.write_str("auto ")?;
                l.to_css(dest)
            },
        }
    }
}

/// A block ellipsis for line clamping.
#[derive(
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum BlockEllipsis {
    /// Display the ellipsis character.
    Ellipsis,
    /// Do not display an ellipsis.
    NoEllipsis,
    /// Display the given string.
    String(crate::values::AtomString),
}

impl BlockEllipsis {
    /// Returns whether this value is `ellipsis`.
    pub fn is_ellipsis(&self) -> bool {
        matches!(self, Self::Ellipsis)
    }
}

/// A keyword for the maximum lines used by `line-clamp`.
#[derive(
    Animate,
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum MaxLinesKeyword {
    /// No block-size clamping is applied.
    None,
    /// Clamp using the available block size.
    Auto,
}

impl MaxLinesKeyword {
    /// Returns whether this keyword is `none`.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// A generic value for the maximum lines used by `line-clamp`.
/// <https://drafts.csswg.org/css-overflow-4/#propdef-max-lines>
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(C)]
#[typed(todo_derive_fields)]
pub struct GenericMaxLines<I> {
    /// The maximum number of lines.
    pub lines: Optional<I>,
    /// Whether block-size clamping is applied.
    #[css(skip_if = "MaxLinesKeyword::is_none")]
    #[animation(constant)]
    pub kw: MaxLinesKeyword,
}

pub use self::GenericMaxLines as MaxLines;

impl<I> MaxLines<I> {
    /// Returns a new `none` value.
    pub fn none() -> Self {
        Self {
            lines: Optional::None,
            kw: MaxLinesKeyword::None,
        }
    }

    /// Returns a new `auto` value.
    pub fn auto() -> Self {
        Self {
            lines: Optional::None,
            kw: MaxLinesKeyword::Auto,
        }
    }

    /// Returns a new line-count value, optionally with the `auto` keyword.
    pub fn lines(lines: I, auto: bool) -> Self {
        Self {
            lines: Optional::Some(lines),
            kw: if auto {
                MaxLinesKeyword::Auto
            } else {
                MaxLinesKeyword::None
            },
        }
    }

    /// Returns whether there is no line-count or block-size clamping.
    pub fn is_none(&self) -> bool {
        self.lines.is_none() && matches!(self.kw, MaxLinesKeyword::None)
    }

    /// Returns whether this value is `auto`.
    pub fn is_auto(&self) -> bool {
        self.lines.is_none() && matches!(self.kw, MaxLinesKeyword::Auto)
    }

    /// Returns the line count if exists.
    pub fn lines_value(&self) -> Option<&I> {
        self.lines.as_ref()
    }
}

/// A generic value for the `line-clamp` property.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(C)]
#[typed(todo_derive_fields)]
#[value_info(other_values = "none")]
pub struct GenericLineClamp<I> {
    /// Maximum `line-count` and `block-size` clamping behavior.
    pub max_lines: GenericMaxLines<I>,
    /// Ellipsis displayed at the clamp point.
    #[animation(constant)]
    pub block_ellipsis: BlockEllipsis,
    /// Whether legacy line clamping behavior is used.
    #[animation(constant)]
    pub webkit_legacy: bool,
}

pub use self::GenericLineClamp as LineClamp;

impl<I> LineClamp<I> {
    /// Returns the `none` value.
    pub fn none() -> Self {
        Self {
            max_lines: MaxLines::none(),
            block_ellipsis: BlockEllipsis::Ellipsis,
            webkit_legacy: false,
        }
    }

    /// Returns whether we are the `none` value or not.
    pub fn is_none(&self) -> bool {
        self.max_lines.is_none() && self.block_ellipsis.is_ellipsis()
    }
}

impl<I: ToCss> ToCss for LineClamp<I> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_none() {
            return dest.write_str("none");
        }

        let mut writer = SequenceWriter::new(dest, " ");
        if !self.max_lines.is_none() {
            writer.item(&self.max_lines)?;
        }
        if !self.block_ellipsis.is_ellipsis() {
            writer.item(&self.block_ellipsis)?;
        }
        if self.webkit_legacy && crate::pref!("layout.css.line-clamp.enabled") {
            writer.raw_item("-webkit-legacy")?;
        }
        Ok(())
    }
}

/// A generic value for the `perspective` property.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(C, u8)]
pub enum GenericPerspective<NonNegativeLength> {
    /// A non-negative length.
    Length(NonNegativeLength),
    /// The keyword `none`.
    None,
}

pub use self::GenericPerspective as Perspective;

impl<L> Perspective<L> {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        Perspective::None
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum PositionProperty {
    Static = 0,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl PositionProperty {
    /// Is the box absolutely positioned?
    pub fn is_absolutely_positioned(self) -> bool {
        matches!(self, Self::Absolute | Self::Fixed)
    }
}

/// https://drafts.csswg.org/css-overflow-4/#overflow-clip-margin's <visual-box>. Note that the
/// spec has special behavior for the omitted keyword, but that's rather odd, see:
/// https://github.com/w3c/csswg-drafts/issues/13185
#[allow(missing_docs)]
#[derive(
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(u8)]
pub enum OverflowClipMarginBox {
    ContentBox,
    PaddingBox,
    BorderBox,
}

/// https://drafts.csswg.org/css-overflow-4/#overflow-clip-margin
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToAnimatedZero,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(C)]
pub struct GenericOverflowClipMargin<L> {
    /// The offset of the clip.
    pub offset: L,
    /// The box that we're clipping to.
    #[animation(constant)]
    pub visual_box: OverflowClipMarginBox,
}

pub use self::GenericOverflowClipMargin as OverflowClipMargin;

impl<L: Zero> GenericOverflowClipMargin<L> {
    /// Returns the `none` value.
    pub fn zero() -> Self {
        Self {
            offset: Zero::zero(),
            visual_box: OverflowClipMarginBox::PaddingBox,
        }
    }
}

impl<L: Zero + ToCss> ToCss for OverflowClipMargin<L> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.visual_box == OverflowClipMarginBox::PaddingBox {
            return self.offset.to_css(dest);
        }
        self.visual_box.to_css(dest)?;
        if !self.offset.is_zero() {
            dest.write_char(' ')?;
            self.offset.to_css(dest)?;
        }
        Ok(())
    }
}

/// The two insets of one scrollbar, for the chrome-only
/// `-moz-scrollbar-inset-block` / `-moz-scrollbar-inset-inline`. Each property
/// names the axis the scrollbar runs along, so `start` and `end` are the logical
/// start and end of that axis and flip with the writing mode.
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(C)]
pub struct GenericScrollbarInset<L> {
    /// The inset at the logical start of the axis.
    pub start: L,
    /// The inset at the logical end of the axis.
    pub end: L,
}

pub use self::GenericScrollbarInset as ScrollbarInset;

impl<L: Zero> ScrollbarInset<L> {
    /// Returns the initial, all-zero value.
    pub fn zero() -> Self {
        Self {
            start: Zero::zero(),
            end: Zero::zero(),
        }
    }
}

impl<L: PartialEq + ToCss> ToCss for ScrollbarInset<L> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.start.to_css(dest)?;
        if self.end != self.start {
            dest.write_char(' ')?;
            self.end.to_css(dest)?;
        }
        Ok(())
    }
}
