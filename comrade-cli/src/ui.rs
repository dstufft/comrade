use tui::backend::Backend;
use tui::layout::{Constraint, Corner, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Tabs};
use tui::Frame;
use tui_logger::{TuiLoggerSmartWidget, TuiWidgetState};

use crate::app::{App, EventsTab, LogsTab};

pub(crate) fn init_logger_state() -> TuiWidgetState {
    TuiWidgetState::new().set_default_display_level(log::LevelFilter::Debug)
}

pub(crate) fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    let titles = app
        .tabs()
        .titles()
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(app.title()))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs().index());

    f.render_widget(tabs, chunks[0]);

    match app.tabs().current().id() {
        "events" => draw_events_tab(f, app, chunks[1]),
        "logs" => draw_logs_tab(f, app, chunks[1]),
        _ => {}
    }
}

fn draw_events_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let tab: &EventsTab = app.tabs().tab("events").expect("could not find events tab");

    let block = Block::default().borders(Borders::ALL);
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = tab.messages().into_iter().map(ListItem::new).collect();
    let list = List::new(items)
        .block(Block::default().title("Messages").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .start_corner(Corner::BottomLeft);

    f.render_widget(list, inner_area);
}

fn draw_logs_tab<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let tab: &LogsTab = app.tabs().tab("logs").expect("could not find logs tab");

    let block = Block::default().borders(Borders::ALL);
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let tui_sm = TuiLoggerSmartWidget::default()
        .title_target("Targets")
        .title_log("Logs")
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::White))
        .style_debug(Style::default().fg(Color::Blue))
        .style_trace(Style::default().fg(Color::DarkGray))
        .output_separator(' ')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_target(true)
        .output_file(false)
        .output_line(false)
        .state(&*tab.state());
    f.render_widget(tui_sm, inner_area);
}
