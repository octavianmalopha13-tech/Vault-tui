use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, AppState};

pub fn draw(f: &mut Frame, app: &App) {
    match app.state {
        AppState::Unlock => draw_unlock(f, app),
        AppState::List => draw_list(f, app),
        AppState::Detail => draw_detail(f, app),
        AppState::ConfirmDelete => draw_confirm(f, app),
        AppState::AddForm => draw_add_form(f, app),
        AppState::Quit => {}
    }
}

fn draw_unlock(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Percentage(40),
        ])
        .split(f.area());

    let masked: String = "*".repeat(app.unlock_input.chars().count());
    let input = Paragraph::new(masked).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Master password — {}", app.vault_path)),
    );
    f.render_widget(input, chunks[1]);

    if let Some(err) = &app.unlock_error {
        let e = Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red));
        f.render_widget(e, chunks[2]);
    }

    let hint = Paragraph::new("Enter to unlock  ·  Esc to quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, chunks[3]);
}

fn draw_list(f: &mut Frame, app: &App) {
    let entries = app
        .vault
        .as_ref()
        .map(|v| v.entries.as_slice())
        .unwrap_or(&[]);

    let items: Vec<ListItem> = entries
        .iter()
        .map(|e| ListItem::new(format!("{}  ({})", e.site, e.user)))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Entries"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !entries.is_empty() {
        state.select(Some(app.selected));
    }

    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(3), Constraint::Length(1)])
    .split(f.area());

    f.render_stateful_widget(list, chunks[0], &mut state);

    if let Some(status) = &app.status {
       let line = Paragraph::new(status.as_str())
        .style(Style::default().fg(Color::Yellow));
       f.render_widget(line, chunks[1]);
   }
    //f.render_stateful_widget(list, f.area(), &mut state);
   // f.render_stateful_widget(list, f.area(), &mut state);
}

fn draw_detail(f: &mut Frame, app: &App) {
    let body = match app.current_entry() {
        Some(e) => format!(
            "site:     {}\nuser:     {}\npassword: {}\nnotes:    {}\n\nEsc to go back",
            e.site, e.user, e.password, e.notes
        ),
        None => "No entry selected.\n\nEsc to go back".into(),
    };

    let p = Paragraph::new(body).block(Block::default().borders(Borders::ALL).title("Detail"));
    f.render_widget(p, f.area());
}

fn draw_confirm(f: &mut Frame, app: &App) {
    let site = app
        .current_entry()
        .map(|e| e.site.clone())
        .unwrap_or_default();

    let body = format!(
        "Delete '{}'?\n\n[y] yes   [n] no",
        site
    );

    let p = Paragraph::new(body).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Confirm delete")
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(p, f.area());
}

fn draw_add_form(f: &mut Frame, app: &App) {
    let labels = ["site", "user", "password", "notes"];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(f.area());

    let title = Paragraph::new("New entry")
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title, chunks[0]);

    for i in 0..4 {
        // mask the password field
        let content = if i == 2 {
            "*".repeat(app.form.fields[i].chars().count())
        } else {
            app.form.fields[i].clone()
        };

        let style = if i == app.form.focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let border_style = if i == app.form.focus {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let p = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .title(labels[i])
                .border_style(border_style),
        ).style(style);

        f.render_widget(p, chunks[1 + i]);
    }

    if let Some(err) = &app.form.error {
        let e = Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red));
        f.render_widget(e, chunks[5]);
    } else {
        let hint = Paragraph::new("Tab/↓ next  ·  Shift-Tab/↑ prev  ·  Enter save  ·  Esc cancel")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, chunks[5]);
    }
}

