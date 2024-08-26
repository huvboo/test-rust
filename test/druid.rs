use druid::widget::{Button, Flex, Label};
use druid::{
    AppLauncher, Application, DruidHandler, LocalizedString, PlatformError, Widget, WidgetExt,
    WindowDesc, WindowHandle,
};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = 0_u32;
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

fn ui_builder() -> impl Widget<u32> {
    // The label text will be computed dynamically based on the current locale and count
    let text =
        LocalizedString::new("hello-counter").with_arg("count", |data: &u32, _env| (*data).into());
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("加")
        .on_click(|_ctx, data, _env| *data += 1)
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)
}

extern crate geo;
extern crate geo_booleanop;

use geo::{BooleanOps, Point};
use geo_types::{LineString, Polygon};

fn main() {
    // let mut reader =
    // shapefile::Reader::from_path(Path::new("C:\\Users\\Administrator\\Desktop\\合并.shp"))
    //     .unwrap();

    let result = shapefile::read_shapes_as::<_, shapefile::PolygonZ>(
        "C:\\Users\\Administrator\\Desktop\\25wpolygon.shp",
    );

    let mut i = 0;
    match result {
        Ok(polygons) => {
            for polygon in polygons {
                let mut points1: Vec<Point> = Vec::new();
                let mut points2: Vec<Point> = Vec::new();
                let mut j = 0;
                for p in polygon.ring(0).unwrap().points() {
                    if j == 0 {
                        points1.push(Point::new(p.x, p.y));
                        points2.push(Point::new(p.x, p.y));
                        j = j + 1;
                    } else if (p.x - points1[j - 1].x()).abs() > 1e-6
                        && (p.y - points1[j - 1].y()).abs() > 1e-6
                    {
                        points1.push(Point::new(p.x, p.y));
                        points2.push(Point::new(p.x, p.y));
                        j = j + 1;
                    }
                }

                if points1.len() > 3 {
                    let poly1: geo::Polygon<f64> = Polygon::new(LineString::from(points1), vec![]);
                    let poly2: geo::Polygon<f64> = Polygon::new(LineString::from(points2), vec![]);

                    println!("{i}, intersect: {:?}", poly2);
                    poly1.intersection(&poly2);
                }

                i = i + 1;
            }
        }
        Err(_) => {}
    }
}
