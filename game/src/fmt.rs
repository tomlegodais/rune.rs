use std::fmt;

use owo_colors::OwoColorize;
use tracing::{
    Event, Level, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

pub struct Formatter;

impl<S, N> FormatEvent<S, N> for Formatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(&self, _ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> fmt::Result {
        let meta = event.metadata();

        let level = match *meta.level() {
            Level::ERROR => format!("{}", "ERROR".red().bold()),
            Level::WARN => format!("{}", "WARN ".yellow().bold()),
            Level::INFO => format!("{}", "INFO ".green().bold()),
            Level::DEBUG => format!("{}", "DEBUG".blue().bold()),
            Level::TRACE => format!("{}", "TRACE".purple().bold()),
        };

        let mut visitor = Visitor::default();
        event.record(&mut visitor);

        write!(writer, "{level} {}  {}", meta.target().dimmed(), visitor.message.bold())?;

        for (k, v) in &visitor.fields {
            write!(writer, "  {}={}", k.cyan(), v.yellow())?;
        }

        writeln!(writer)
    }
}

#[derive(Default)]
struct Visitor {
    message: String,
    fields: Vec<(String, String)>,
}

impl Visit for Visitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.fields.push((field.name().to_owned(), value.to_owned()));
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.fields.push((field.name().to_owned(), format!("{value:?}")));
        }
    }
}
