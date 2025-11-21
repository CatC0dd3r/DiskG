mod file_module;

use std::{io, env};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::{Alignment, Constraint, Direction, Layout, Modifier, Style, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

struct App {
    items: Vec<String>,
    state: ListState,
    flag: String
}


impl App {
    fn new(items: Vec<String>, flag: String) -> App {
        let mut state = ListState::default();
        state.select(Some(0)); 
        App { items, state, flag }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }
}


fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {            
        println!("Помощь: ./DiskG -h");
        return Ok(())
    }

    let flag = &args[1];

    let app_items;

    match flag.as_str() {
        "-d" => {
            use file_module::get_partions;
            app_items = get_partions();
        },
        "-f" => {
            use file_module::get_files_and_dir;
            app_items = get_files_and_dir();
        },
        "-h" => {
            println!("Использование: ./DiskG [опция]\n-d -> Просмотр информации о разделах\n-f -> Просмотр информации о файлах/каталогах внутри каталога, из которого запущена утилита\nИспользуйте Q, чтобы выйти");
            return Ok(())
        },
        &_ => {
            println!("Помощь: ./DiskG -h");
            return Ok(())
        }
    }

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(app_items?, flag.to_string());

    let _res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}


fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f|{
            ui(f, app);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Up => app.previous(),
                KeyCode::Down => app.next(),
                _ => {}
            }
        }
    }
}


fn ui(f: &mut Frame<'_>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ].as_ref())
        .split(f.size());


    let items: Vec<ListItem> = app.items.iter()
        .map(|i| ListItem::new(Text::from(i.as_str())))
        .collect();

    let items_list = List::new(items)
        .block(Block::default().title("Файлы/Разделы").title_alignment(Alignment::Center).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_list, chunks[0], &mut app.state);
    
    let detail_text = if let Some(selected_index) = app.state.selected() {
        if let Some(selected_item_name) = app.items.get(selected_index) {
            match app.flag.as_str() { 
                "-d" => {
                    use file_module::disk_check;
                    match disk_check(selected_item_name) {
                        Ok(info) => info,
                        Err(e) => format!("Ошибка диска: {}", e)
                    }
                },
                "-f" => {
                    use file_module::get_size;
                    match get_size(selected_item_name) {
                        Ok(info) => info,
                        Err(e) => format!("Ошибка файла/папки: {}", e)
                    }
                },
                _ => String::from("Неизвестный режим.")
            }
        } else {
            String::from("Нет информации о выбранном элементе.")
        }
    } else {
        String::from("Элемент не выбран.")
    };
    
    let right_panel = Paragraph::new(Text::from(detail_text))
        .block(Block::default()
            .title("Информация о блоках")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
        );

    f.render_widget(right_panel, chunks[1])
}
