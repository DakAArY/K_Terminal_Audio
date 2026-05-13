use ratatui::{
    Frame, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Modifier, Style}, symbols, widgets::{Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph}
};
use crate::app::App;
use ratatui_image::{StatefulImage};

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(12),    
            Constraint::Min(10),      
            Constraint::Length(3),     
        ])
        .split(size);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),    
            Constraint::Percentage(50),   
        ])
        .split(chunks[1]);

    if let Some(protocol) = &mut app.cover_art {
        let block = Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Black));
        let inner_area = block.inner(header_chunks[0]);
        
        f.render_widget(block, header_chunks[0]);

        let fondo_negro = Block::default().style(Style::default().bg(Color::Black));
        f.render_widget(fondo_negro, inner_area);

        let image_widget = StatefulImage::default();
        f.render_stateful_widget(image_widget, inner_area, protocol);
    } else {
       let fallback = Paragraph::new("\n\n\n K T A\n[NO ART]") 
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
       f.render_widget(fallback, header_chunks[0]); 
    }

    
    let header = Paragraph::new(format!(" |> {} ", app.current_track_name))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" Reproduciendo "));

    
    let items: Vec<ListItem> = app.items
        .iter()
        .map(|track| ListItem::new(format!("  * {}", track.display_name)))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Biblioteca "))
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    match app.vis_mode {
        crate::app::VisMode::Spectrum => {
            let bars: Vec<Bar> = app.fft_bars.iter().map(|&v| Bar::default().value(v).text_value("")).collect();
            let barchart = BarChart::default()
                .block(Block::default().borders(Borders::ALL))
                .data(BarGroup::default().bars(&bars))
                .bar_width(3)
                .bar_gap(1)
                .bar_style(Style::default().fg(Color::Red));
            f.render_widget(barchart, body_chunks[1]);
        }
        crate::app::VisMode::Oscilloscope => {
            let datasets = vec![
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Red))
                    .data(&app.oscilloscope_data),
            ];

            let chart = Chart::new(datasets)
                .block(Block::default().borders(Borders::ALL))
                .x_axis(Axis::default().bounds([0.0, 1024.0]))
                .y_axis(Axis::default().bounds([-1.0, 1.0]));
            f.render_widget(chart, body_chunks[1]);
        }
    }
    
    let loop_status = if app.loop_track { "ON " } else { "OFF" };
    let controls_text = format!(
        " [Espacio] Pausa | [p/←] Anterior | [n/→] Siguiente | [l] Bucle: {} | [v] Vista | [q] Salir",
        loop_status
    );
    let help = Paragraph::new(controls_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(if app.loop_track { Color::Green } else { Color::Yellow }))
        .block(Block::default().borders(Borders::ALL)); 
    
    f.render_widget(header, header_chunks[1]);
    f.render_stateful_widget(list, body_chunks[0], &mut app.state); 
    f.render_widget(help, chunks[2]);
}
