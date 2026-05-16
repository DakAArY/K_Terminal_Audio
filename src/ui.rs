use crate::app::{App, VisMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{
        Axis, Bar, BarChart, BarGroup, Block, Borders, BorderType, Chart, Clear, Dataset, GraphType, List,
        ListItem, Paragraph,
    },
};
use ratatui_image::StatefulImage;

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();

    f.render_widget(Clear, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(11),  
            Constraint::Min(12),    
            Constraint::Length(2),  
        ])
        .split(size);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(0)]) 
        .split(chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)]) 
        .split(chunks[1]);

    if let Some(protocol) = &mut app.cover_art {
        let block = Block::default()
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Red))
            .border_type(BorderType::Thick);
        let inner_area = block.inner(header_chunks[0]);

        f.render_widget(block, header_chunks[0]);
        let image_widget = StatefulImage::default();
        f.render_stateful_widget(image_widget, inner_area, protocol);
    } else {
        let fallback_text = "\n\n\n\n ◤ KTA ◢\n NO ART";
        let fallback = Paragraph::new(fallback_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)) 
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray))
            );
        f.render_widget(fallback, header_chunks[0]);
    }

    let current_track_info = if let Some(i) = app.state.selected() {
        app.items.get(i)
    } else {
        None
    };

    let header_text = if let Some(track) = current_track_info {
        let mins = track.duration_secs / 60;
        let secs = track.duration_secs % 60;
        format!(
            "\n ◤ TRACK: {} ◢\n    ALBUM: {}\n    TIME: {:02}:{:02}  //  BR: {} kbps",
            track.display_name.to_uppercase(), track.album.to_uppercase(), mins, secs, track.bitrate
        )
    } else {
        format!("\n ◤ SYSTEM: {} ◢", app.current_track_name.to_uppercase())
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Gray)) 
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Red))
                .border_type(BorderType::Thick)
                .title(" [ STATUS: ACTIVE ] ")
                .title_style(Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)) 
        );
    f.render_widget(header, header_chunks[1]);


    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let mins = track.duration_secs / 60;
            let secs = track.duration_secs % 60;
            let is_selected = app.state.selected() == Some(index);

            if is_selected {
                ListItem::new(format!(
                    " ► {} [{:02}:{:02}] ◄ ",
                    track.display_name.to_uppercase(), mins, secs
                ))
                .style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(format!(
                    "   ○ {} [{:02}:{:02}]",
                    track.display_name, mins, secs
                ))
                .style(Style::default().fg(Color::Gray))
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(Color::Red))
                .border_type(BorderType::Thick)
                .title(" ◤ ARCHIVE ◢ ")
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        );
    f.render_stateful_widget(list, body_chunks[0], &mut app.state);


    match app.vis_mode {
        VisMode::Spectrum => {
            let vis_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" [ FREQUENCY SPECTRUM ] ")
                .title_style(Style::default().fg(Color::Gray));
            
            let inner_area = vis_block.inner(body_chunks[1]);
            f.render_widget(vis_block, body_chunks[1]);

            let bars: Vec<Bar> = app
                .fft_bars
                .iter()
                .map(|&v| {
                    let bar_color = if v > 80 { Color::LightRed } else { Color::Red };
                    Bar::default().value(v).text_value("").style(Style::default().fg(bar_color))
                })
                .collect();

            let available_width = inner_area.width; 
            let num_bars = app.fft_bars.len() as u16;
            
            let unit_width = if num_bars > 0 { available_width / num_bars } else { 1 };
            let dynamic_gap = if unit_width > 2 { 1 } else { 0 };
            let dynamic_width = unit_width.saturating_sub(dynamic_gap).max(1);
            
            let total_chart_width = num_bars * (dynamic_width + dynamic_gap);
            let pad_left = available_width.saturating_sub(total_chart_width) / 2;

            let center_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(pad_left),
                    Constraint::Min(0)
                ])
                .split(inner_area);

            let barchart = BarChart::default()
                .data(BarGroup::default().bars(&bars))
                .bar_width(dynamic_width) 
                .bar_gap(dynamic_gap);
                
            f.render_widget(barchart, center_layout[1]);
        }
        VisMode::Oscilloscope => {
            let vis_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" [ WAVEFORM OSCILLOSCOPE ] ")
                .title_style(Style::default().fg(Color::Gray));

            let inner_area = vis_block.inner(body_chunks[1]);
            f.render_widget(vis_block, body_chunks[1]);

            let datasets = vec![
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)) 
                    .data(&app.oscilloscope_data),
            ];

            let chart = Chart::new(datasets)
                .x_axis(Axis::default().bounds([0.0, 1024.0]).style(Style::default().fg(Color::DarkGray)))
                .y_axis(Axis::default().bounds([-1.0, 1.0]).style(Style::default().fg(Color::DarkGray)));
                
            f.render_widget(chart, inner_area);
        }
    }

    let loop_status = if app.loop_track { "LOOP: ON " } else { "LOOP: OFF" };
    let controls_text = format!(
        " [SPACE] PAUSE // [P] PREV // [N] NEXT // [L] {} // [V] VISUAL // [Q] ABORT ",
        loop_status
    );

    let (footer_bg, footer_fg) = if app.loop_track {
        (Color::Gray, Color::Black) 
    } else {
        (Color::Red, Color::Black)
    };

    let help = Paragraph::new(controls_text)
        .alignment(Alignment::Center)
        .style(Style::default().bg(footer_bg).fg(footer_fg).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(help, chunks[2]);
}
