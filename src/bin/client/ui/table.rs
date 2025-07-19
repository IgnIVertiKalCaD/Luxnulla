use std::io::{self, Result};
use std::time::{Duration, Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, TableState},
};

#[derive(Debug, Clone)]
struct SubData {
    id: u32,
    protocol: String,
    address: String,
    name: String,
    test_result: String,
}

impl SubData {
    fn new(id: u32) -> Self {
        Self {
            id,
            protocol: String::from(""),
            address: String::from(""),
            name: String::from(""),
            test_result: String::from(""),
        }
    }
}

struct App {
    state: TableState,
    items: Vec<SubData>,
    last_update: Instant,
    scroll_offset: usize,
    visible_rows: usize,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            state: TableState::default(),
            items: Vec::new(),
            last_update: Instant::now(),
            scroll_offset: 0,
            visible_rows: 10,
        };

        app.state.select(Some(0));
        app
    }

    // fn generate_data(&mut self) {
    //     self.items.clear();
    //     // Генерируем больше данных, чтобы показать скролл
    //     for i in 0..150 {
    //         self.items.push(SubData::new(i + 1));
    //     }
    // }

    fn next(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let next_index = if selected >= self.items.len() - 1 {
            0
        } else {
            selected + 1
        };

        self.state.select(Some(next_index));

        // Обновляем скролл если необходимо
        if next_index >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = next_index - self.visible_rows + 1;
        } else if next_index < self.scroll_offset {
            self.scroll_offset = next_index;
        }
    }

    fn previous(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let prev_index = if selected == 0 {
            self.items.len() - 1
        } else {
            selected - 1
        };

        self.state.select(Some(prev_index));

        // Обновляем скролл если необходимо
        if prev_index < self.scroll_offset {
            self.scroll_offset = prev_index;
        } else if prev_index >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = prev_index - self.visible_rows + 1;
        }
    }

    fn page_down(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let new_index = (selected + self.visible_rows).min(self.items.len() - 1);
        self.state.select(Some(new_index));

        if new_index >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = new_index - self.visible_rows + 1;
        }
    }

    fn page_up(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let new_index = if selected >= self.visible_rows {
            selected - self.visible_rows
        } else {
            0
        };
        self.state.select(Some(new_index));

        if new_index < self.scroll_offset {
            self.scroll_offset = new_index;
        }
    }

    fn update_data(&mut self) {
        if self.last_update.elapsed() >= Duration::from_secs(3) {
            // Обновляем только значения и время, оставляем структуру
            let mut rng = rand::thread_rng();
            let statuses = [
                "Active",
                "Inactive",
                "Pending",
                "Complete",
                "Error",
                "Processing",
                "Waiting",
            ];

            // for item in &mut self.items {
            //     item.value = rng.gen_range(0.0..1000.0);
            //     item.timestamp = chrono::Utc::now().format("%H:%M:%S").to_string();
            //     if rng.gen_bool(0.2) {
            //         // 20% шанс изменить статус
            //         item.status = statuses[rng.gen_range(0..statuses.len())].to_string();
            //     }
            // }

            self.last_update = Instant::now();
        }
    }

    fn update_visible_rows(&mut self, height: usize) {
        // Высота таблицы = общая высота - заголовок - рамки - панель помощи
        // Вычитаем 6: 2 для рамок таблицы, 1 для заголовка, 3 для панели помощи
        self.visible_rows = if height > 6 { height - 6 } else { 1 };
    }

    fn get_visible_items(&self) -> Vec<&SubData> {
        let end = (self.scroll_offset + self.visible_rows).min(self.items.len());
        self.items[self.scroll_offset..end].iter().collect()
    }

    fn get_scroll_progress(&self) -> f64 {
        if self.items.len() <= self.visible_rows {
            1.0
        } else {
            self.scroll_offset as f64 / (self.items.len() - self.visible_rows) as f64
        }
    }

    fn get_scroll_thumb_position(&self, scrollbar_height: usize) -> usize {
        if self.items.len() <= self.visible_rows {
            0
        } else {
            let progress = self.get_scroll_progress();
            ((scrollbar_height as f64 - 1.0) * progress).round() as usize
        }
    }
}

pub fn init() -> Result<()> {
    // Настройка терминала
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Создаем приложение
    let mut app = App::new();

    // Основной цикл
    let res = run_app(&mut terminal, &mut app);

    // Восстанавливаем терминал
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Обновляем данные каждые 3 секунды
        app.update_data();

        // Обрабатываем события
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::PageDown => app.page_down(),
                        KeyCode::PageUp => app.page_up(),
                        KeyCode::Home => {
                            app.state.select(Some(0));
                            app.scroll_offset = 0;
                        }
                        KeyCode::End => {
                            let last_index = app.items.len() - 1;
                            app.state.select(Some(last_index));
                            app.scroll_offset = if last_index >= app.visible_rows {
                                last_index - app.visible_rows + 1
                            } else {
                                0
                            };
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    // Обновляем количество видимых строк на основе размера терминала
    app.update_visible_rows(f.size().height as usize);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(f.size());

    // Разделяем верхнюю часть на таблицу и скролл-бар
    let table_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(main_layout[0]);

    let header_cells = ["ID", "Name", "Status", "Value", "Time"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1);

    // Получаем только видимые элементы
    let visible_items = app.get_visible_items();
    let rows = visible_items.iter().enumerate().map(|(i, item)| {
        let actual_index = app.scroll_offset + i;
        let cells = vec![
            Cell::from(format!("{:03}", item.id)),
            Cell::from(item.name.clone()),
            // Cell::from(item.status.clone()).style(match item.status.as_str() {
            //     "Active" => Style::default()
            //         .fg(Color::Green)
            //         .add_modifier(Modifier::BOLD),
            //     "Error" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            //     "Pending" => Style::default()
            //         .fg(Color::Yellow)
            //         .add_modifier(Modifier::BOLD),
            //     "Processing" => Style::default()
            //         .fg(Color::Cyan)
            //         .add_modifier(Modifier::BOLD),
            //     "Complete" => Style::default()
            //         .fg(Color::Magenta)
            //         .add_modifier(Modifier::BOLD),
            //     "Waiting" => Style::default()
            //         .fg(Color::Blue)
            //         .add_modifier(Modifier::BOLD),
            //     _ => Style::default().fg(Color::Gray),
            // }),
            Cell::from(format!("{}", item.protocol)),
            Cell::from(format!("{}", item.address)),
        ];
        Row::new(cells).height(1)
    });

    // Создаем заголовок с информацией о скролле
    let selected = app.state.selected().unwrap_or(0);
    let title = format!("🚀 Demo Table ({}/{})", selected + 1, app.items.len());

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
    )
    .highlight_style(
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Yellow),
    )
    .highlight_symbol(">> ");

    // Создаем состояние для отображения выделения относительно видимых элементов
    let mut display_state = TableState::default();
    if let Some(selected_index) = app.state.selected() {
        if selected_index >= app.scroll_offset
            && selected_index < app.scroll_offset + app.visible_rows
        {
            display_state.select(Some(selected_index - app.scroll_offset));
        }
    }

    f.render_stateful_widget(table, table_layout[0], &mut display_state);

    // Рендерим скролл-бар
    render_scrollbar(f, app, table_layout[1]);

    let help_message = Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to quit, ", Style::default().fg(Color::Gray)),
        Span::styled(
            "↑/↓",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to navigate, ", Style::default().fg(Color::Gray)),
        Span::styled(
            "PgUp/PgDn",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to scroll, ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Home/End",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to jump, ", Style::default().fg(Color::Gray)),
        Span::styled(
            "r",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to refresh", Style::default().fg(Color::Gray)),
    ]);

    let help = ratatui::widgets::Paragraph::new(help_message)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(Alignment::Center);

    f.render_widget(help, main_layout[1]);
}

fn render_scrollbar(f: &mut Frame, app: &App, area: Rect) {
    if app.items.len() <= app.visible_rows {
        // Если все элементы помещаются, показываем пустой скролл-бар
        let scrollbar = Block::default()
            .borders(Borders::ALL)
            .title("━")
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(scrollbar, area);
        return;
    }

    // Вычисляем размеры скролл-бара
    let scrollbar_height = area.height.saturating_sub(2) as usize; // Вычитаем рамки
    let thumb_position = app.get_scroll_thumb_position(scrollbar_height);
    let thumb_size = (scrollbar_height * app.visible_rows / app.items.len()).max(1);

    // Создаем содержимое скролл-бара
    let mut scrollbar_content = Vec::new();
    for i in 0..scrollbar_height {
        if i >= thumb_position && i < thumb_position + thumb_size {
            // Ползунок скролла
            scrollbar_content.push(Line::from(Span::styled(
                "█",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            // Фон скролла
            scrollbar_content.push(Line::from(Span::styled(
                "░",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Информация о позиции
    let position_info = format!(
        "{}-{}",
        app.scroll_offset + 1,
        (app.scroll_offset + app.visible_rows).min(app.items.len())
    );

    let scrollbar = Paragraph::new(scrollbar_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(position_info)
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray)),
        )
        .alignment(Alignment::Center);

    f.render_widget(scrollbar, area);
}
