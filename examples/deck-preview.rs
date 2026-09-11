//! Capture the production card painter on a native window, without touching saves.
use eframe::egui::{self, pos2, vec2, Color32, Rect};
use omarchy_solitaire::{
    cards::{self, DeckStyle},
    deck::DeckArt,
    game::Card,
    theme::Palette,
};
struct Preview {
    deck: DeckArt,
    palette: Palette,
    output: String,
    all: bool,
    numbers: bool,
    capture: bool,
}
impl eframe::App for Preview {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(self.palette.table)).show(ctx,|ui|{
            let p=ui.painter();
            let style=|pattern|DeckStyle{palette:&self.palette,pattern,holographic:false,sheen:0.5,deck:&self.deck,art:None};
            let label=|x,y,t:&str,size|{p.text(pos2(x,y),egui::Align2::LEFT_TOP,t,egui::FontId::proportional(size),self.palette.text);};
            if self.numbers {
                label(32.,20.,"OMARCHY SOLITAIRE · Number-card study",26.);
                label(32.,60.,"Measured rank spacing · natural-width 10 · balanced pip field",16.);
                for (n,id) in [1,15,30,45,7,34,48].iter().enumerate() {
                    let r=Rect::from_min_size(pos2(32.+n as f32*131.,110.),vec2(116.,162.4));
                    cards::paint(p,r,Card(*id,true),&style(0),false);
                }
                label(32.,310.,"Overlapping columns at playing size",20.);
                for (n,width) in [80.,96.,112.,124.].iter().enumerate() {
                    let x=32.+n as f32*228.;
                    label(x,349.,&format!("{width} px"),16.);
                    for (row,id) in [51,24,10,35,8,33].iter().enumerate() {
                        let r=Rect::from_min_size(pos2(x,386.+row as f32*width*0.28),vec2(*width,width*1.4));
                        cards::paint(p,r,Card(*id,true),&style(0),false);
                    }
                }
            } else if self.all {
                label(22.,14.,"OMARCHY SOLITAIRE · Complete deck / 80 px",22.);
                for n in 0..52 { let r=Rect::from_min_size(pos2(22.+(n%13) as f32*91.,62.+(n/13) as f32*143.),vec2(80.,112.));cards::paint(p,r,Card(n as u8,true),&style(0),false); }
            } else {
                label(34.,20.,"OMARCHY SOLITAIRE",28.);
                label(34.,59.,"Production deck · rendered by the native game",15.);
                for (n,id) in [39,32,23,11,51].iter().enumerate(){let r=Rect::from_min_size(pos2(34.+n as f32*182.,106.),vec2(160.,224.));cards::paint(p,r,Card(*id,true),&style(0),false);}
                for (n,name) in ["Tilework","Engraved","Foil"].iter().enumerate(){let x=34.+n as f32*300.;let r=Rect::from_min_size(pos2(x,374.),vec2(200.,280.));cards::paint(p,r,Card(0,false),&style(n),false);label(x,665.,name,20.);let small=Rect::from_min_size(pos2(x+210.,520.),vec2(80.,112.));cards::paint(p,small,Card(0,false),&style(n),false);}
                label(34.,718.,"Theme-coloured engraving · exact Omarchy mark · reversible courts and indexes",15.);
            }
        });
        let captured = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let egui::Event::Screenshot { image, .. } = e {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let Some(i) = captured {
            image::save_buffer(
                &self.output,
                &i.pixels
                    .iter()
                    .flat_map(|p| p.to_array())
                    .collect::<Vec<_>>(),
                i.width() as u32,
                i.height() as u32,
                image::ColorType::Rgba8,
            )
            .expect("save preview");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if !self.capture && ctx.input(|i| i.time) > 0.5 {
            self.capture = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        ctx.request_repaint();
    }
}
fn main() -> eframe::Result {
    let args: Vec<_> = std::env::args().collect();
    let numbers = args.iter().any(|x| x == "--numbers");
    let all = args.iter().any(|x| x == "--all");
    let light = args.iter().any(|x| x == "--light");
    let palette = if light {
        Palette::new(
            Color32::from_rgb(236, 233, 223),
            Color32::from_rgb(38, 54, 48),
            Color32::from_rgb(65, 103, 55),
        )
    } else {
        Palette::default()
    };
    let output = args.get(1).expect("output PNG required").clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(if all {
            [1220., 660.]
        } else {
            [960., 770.]
        }),
        ..Default::default()
    };
    eframe::run_native(
        "Solitaire deck proof",
        options,
        Box::new(move |cc| {
            Ok(Box::new(Preview {
                deck: DeckArt::new(&cc.egui_ctx, &palette),
                palette,
                output,
                all,
                numbers,
                capture: false,
            }))
        }),
    )
}
