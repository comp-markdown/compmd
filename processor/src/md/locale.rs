//! The document's language (spec §8.2): dates, numbers, currencies, plurals, lists and the first day of the week by
//! CLDR, through ICU4X and all its compiled locale data.

use fixed_decimal::{Decimal, FloatPrecision, SignedRoundingMode, UnsignedRoundingMode};
use icu::calendar::week::WeekInformation;
use icu::calendar::{Date, Iso};
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

/// A locale, from `md.lang`.
pub(crate) struct Language(Locale);

/// The largest number of fraction digits `md.number` writes.
const MAX_DIGITS: i64 = 20;

impl Language {
    /// The locale of a BCP 47 tag; CLDR's root locale when the tag isn't one.
    pub(crate) fn new(tag: &str) -> Self {
        Self(Locale::try_from_str(tag).unwrap_or(Locale::UNKNOWN))
    }

    /// A date in CLDR's `short`, `medium`, `long` or `full` style.
    pub(crate) fn date(
        &self,
        (year, month, day): (i64, i64, i64),
        style: &str,
    ) -> Result<String, String> {
        let out_of_range = |_| "the date is out of range".to_string();
        let date: Date<Iso> = Date::try_new_iso(
            i32::try_from(year).map_err(out_of_range)?,
            u8::try_from(month).map_err(out_of_range)?,
            u8::try_from(day).map_err(out_of_range)?,
        )
        .map_err(|e| e.to_string())?;
        let prefs = (&self.0).into();
        let formatted = match style {
            "short" => DateTimeFormatter::try_new(prefs, fieldsets::YMD::short())
                .map(|f| f.format(&date).to_string()),
            "medium" => DateTimeFormatter::try_new(prefs, fieldsets::YMD::medium())
                .map(|f| f.format(&date).to_string()),
            "long" => DateTimeFormatter::try_new(prefs, fieldsets::YMD::long())
                .map(|f| f.format(&date).to_string()),
            "full" => DateTimeFormatter::try_new(prefs, fieldsets::YMDE::long())
                .map(|f| f.format(&date).to_string()),
            other => {
                return Err(format!(
                    "md.date() takes the style \"short\", \"medium\", \"long\" or \"full\", not \"{other}\""
                ));
            }
        };
        formatted.map_err(|e| e.to_string())
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
    pub(crate) fn plural(&self, n: f64) -> Result<&'static str, String> {
        let rules = PluralRules::try_new_cardinal((&self.0).into()).map_err(|e| e.to_string())?;
        Ok(match rules.category_for(&decimal(n)?) {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        })
    }

    /// Items joined by the locale's list pattern: for "and", or for alternatives, "or".
    pub(crate) fn list(&self, items: &[String], or: bool) -> Result<String, String> {
        let (prefs, options) = ((&self.0).into(), ListFormatterOptions::default());
        let formatter = if or {
            ListFormatter::try_new_or(prefs, options)
        } else {
            ListFormatter::try_new_and(prefs, options)
        }
        .map_err(|e| e.to_string())?;
        Ok(formatter
            .format(items.iter().map(String::as_str))
            .to_string())
    }

    /// The first day of the week, 1 for Monday to 7 for Sunday.
    pub(crate) fn first_weekday(&self) -> Result<i64, String> {
        let week = WeekInformation::try_new((&self.0).into()).map_err(|e| e.to_string())?;
        Ok(i64::from(week.first_weekday as u8))
    }
}

/// The decimal of `n`, with the shortest digits that read back as it.
fn decimal(n: f64) -> Result<Decimal, String> {
    Decimal::try_from_f64(n, FloatPrecision::RoundTrip)
        .map_err(|_| format!("{n} is not a finite number"))
}
