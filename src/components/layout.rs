enum Layout_Vertical {
    LEFT,
    CENTER,
    RIGHT,
}

enum Layout_Horizontal {
    TOP,
    CENTER,
    BOTTOM,
}

pub struct Layout {
    w: f32, // 宽，px
    h: f32, // 高，px
    x: f32, // 距离窗口左边界，px
    y: f32, // 距离窗口下边界，px
    Layout_Vertical: Layout_Vertical,
    Layout_Horizontal: Layout_Horizontal,
}
