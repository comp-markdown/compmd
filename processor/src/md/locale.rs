//! The document's language (spec §8.2): dates, numbers, currencies, plurals, lists and the first day of the week by
//! CLDR, through ICU4X and all its compiled locale data.

use crate::cel::{Key, Value};
use crate::named::named;
use fixed_decimal::{Decimal, FloatPrecision, SignedRoundingMode, UnsignedRoundingMode};
use icu::calendar::types::Weekday;
use icu::calendar::week::WeekInformation;
use icu::calendar::{Date, Iso};
use icu::datetime::fieldsets::enums::DateFieldSet;
use icu::datetime::{DateTimeFormatter, fieldsets};
use icu::decimal::DecimalFormatter;
use icu::decimal::options::DecimalFormatterOptions;
use icu::list::ListFormatter;
use icu::list::options::ListFormatterOptions;
use icu::locale::Locale;
use icu::plurals::{PluralCategory, PluralRules};
use icu_experimental::dimension::currency::CurrencyType;
use icu_experimental::dimension::currency::formatter::CurrencyFormatter;
use icu_experimental::dimension::currency::options::CurrencyFormatterOptions;
use indexmap::IndexMap;

/// A locale, from `md.lang`.
pub(crate) struct Language(Locale);

/// The largest number of fraction digits `md.number` writes.
const MAX_DIGITS: i64 = 20;

impl Language {
    /// The locale of a BCP 47 tag; CLDR's root locale when the tag isn't one.
    pub(crate) fn new(tag: &str) -> Self {
        Self(Locale::try_from_str(tag).unwrap_or(Locale::UNKNOWN))
    }

    /// A date in one of CLDR's styles.
    pub(crate) fn date(&self, date: Date<Iso>, style: DateStyle) -> Result<String, String> {
        let formatter = DateTimeFormatter::try_new((&self.0).into(), style.fields())
            .map_err(|e| e.to_string())?;
        Ok(formatter.format(&date).to_string())
    }

    /// `n` with `digits` fraction digits, rounded half away from zero as `Intl.NumberFormat` does.
    pub(crate) fn number(&self, n: f64, digits: i64) -> Result<String, String> {
        if !(0..=MAX_DIGITS).contains(&digits) {
            return Err(format!(
                "md.number() takes 0 to {MAX_DIGITS} digits, not {digits}"
            ));
        }
        let position = -i16::try_from(digits).map_err(|e| e.to_string())?;
        let mut decimal = decimal(n)?;
        decimal.round_with_mode(
            position,
            SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfExpand),
        );
        decimal.absolute.pad_end(position);
        let formatter =
            DecimalFormatter::try_new((&self.0).into(), DecimalFormatterOptions::default())
                .map_err(|e| e.to_string())?;
        Ok(formatter.format(&decimal).to_string())
    }

    /// `n` as an amount of the currency `code` (ISO 4217, such as `USD`), with the currency's symbol and digits.
    pub(crate) fn currency(&self, n: f64, code: &str) -> Result<String, String> {
        let currency: CurrencyType = code
            .parse()
            .map_err(|_| format!("'{code}' is not a currency code such as \"USD\""))?;
        let options = CurrencyFormatterOptions::default();
        let formatter = CurrencyFormatter::try_new_symbol((&self.0).into(), currency, options)
            .map_err(|e| e.to_string())?;
        Ok(formatter.format_fixed_decimal(&decimal(n)?).to_string())
    }

    /// CLDR's plural category of `n`.
    fn category(&self, n: f64) -> Result<PluralCategory, String> {
        let rules = PluralRules::try_new_cardinal((&self.0).into()).map_err(|e| e.to_string())?;
        Ok(rules.category_for(&decimal(n)?))
    }

    /// Whether `n` is in the category `one`.
    pub(crate) fn is_one(&self, n: f64) -> Result<bool, String> {
        Ok(self.category(n)? == PluralCategory::One)
    }

    /// The form for `n` among `forms`, keyed by CLDR's plural categories (`one`, `few`, …), falling back to
    /// `other`.
    pub(crate) fn plural_form(
        &self,
        n: f64,
        forms: &IndexMap<Key, Value>,
    ) -> Result<Value, String> {
        let wanted = self.category(n)?;
        let (mut form, mut other) = (None, None);
        for (key, value) in forms {
            let Key::String(name) = key else {
                return Err(format!(
                    "md.plural() takes forms named by plural categories, not {}",
                    key.to_value().text()
                ));
            };
            let category = PluralCategory::get_for_cldr_string(name).ok_or_else(|| {
                format!("md.plural() takes forms named by plural categories, not \"{name}\"")
            })?;
            if category == wanted {
                form = Some(value);
            }
            if category == PluralCategory::Other {
                other = Some(value);
            }
        }
        form.or(other)
            .cloned()
            .ok_or_else(|| "md.plural() needs an \"other\" form".to_string())
    }

    /// Items joined by the locale's list pattern.
    pub(crate) fn list(&self, items: &[String], kind: ListKind) -> Result<String, String> {
        let (prefs, options) = ((&self.0).into(), ListFormatterOptions::default());
        let formatter = match kind {
            ListKind::And => ListFormatter::try_new_and(prefs, options),
            ListKind::Or => ListFormatter::try_new_or(prefs, options),
        }
        .map_err(|e| e.to_string())?;
        Ok(formatter
            .format(items.iter().map(String::as_str))
            .to_string())
    }

    /// The first day of the week.
    pub(crate) fn first_weekday(&self) -> Result<Weekday, String> {
        let week = WeekInformation::try_new((&self.0).into()).map_err(|e| e.to_string())?;
        Ok(week.first_weekday)
    }
}

named! {
    /// CLDR's date styles.
    pub(crate) enum DateStyle {
        Short = "short",
        Medium = "medium",
        Long = "long",
        Full = "full",
    }
}

impl DateStyle {
    /// The fields and length CLDR writes the style with: `full` adds the weekday.
    fn fields(self) -> DateFieldSet {
        match self {
            Self::Short => DateFieldSet::YMD(fieldsets::YMD::short()),
            Self::Medium => DateFieldSet::YMD(fieldsets::YMD::medium()),
            Self::Long => DateFieldSet::YMD(fieldsets::YMD::long()),
            Self::Full => DateFieldSet::YMDE(fieldsets::YMDE::long()),
        }
    }
}

named! {
    /// How `md.list` joins: "a, b, and c", or as alternatives, "a, b, or c".
    pub(crate) enum ListKind {
        And = "and",
        Or = "or",
    }
}

/// The decimal of `n`, with the shortest digits that read back as it.
fn decimal(n: f64) -> Result<Decimal, String> {
    Decimal::try_from_f64(n, FloatPrecision::RoundTrip)
        .map_err(|_| format!("{n} is not a finite number"))
}
