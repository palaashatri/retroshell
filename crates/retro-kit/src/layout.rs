use crate::{Rect, Size, Widget};

#[derive(Debug, Clone, Copy)]
pub struct LayoutConstraint {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl LayoutConstraint {
    pub const UNBOUNDED: LayoutConstraint = LayoutConstraint {
        min_width: 0.0,
        max_width: f32::INFINITY,
        min_height: 0.0,
        max_height: f32::INFINITY,
    };

    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }

    pub fn clamp(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutHints {
    Fill,
    Fit,
    Center,
    Start,
    End,
    Stretch,
}

pub enum Layout {
    Horizontal {
        spacing: f32,
        padding: f32,
        children: Vec<Box<dyn Widget>>,
    },
    Vertical {
        spacing: f32,
        padding: f32,
        children: Vec<Box<dyn Widget>>,
    },
    Grid {
        columns: usize,
        spacing: f32,
        padding: f32,
        children: Vec<Box<dyn Widget>>,
    },
    Stack {
        children: Vec<Box<dyn Widget>>,
    },
    Overlay {
        children: Vec<Box<dyn Widget>>,
    },
}

impl Layout {
    pub fn horizontal(spacing: f32) -> Self {
        Layout::Horizontal {
            spacing,
            padding: 0.0,
            children: vec![],
        }
    }

    pub fn vertical(spacing: f32) -> Self {
        Layout::Vertical {
            spacing,
            padding: 0.0,
            children: vec![],
        }
    }

    pub fn grid(columns: usize, spacing: f32) -> Self {
        Layout::Grid {
            columns,
            spacing,
            padding: 0.0,
            children: vec![],
        }
    }

    pub fn padding(self, padding_val: f32) -> Self {
        match self {
            Layout::Horizontal {
                spacing, children, ..
            } => Layout::Horizontal {
                spacing,
                padding: padding_val,
                children,
            },
            Layout::Vertical {
                spacing, children, ..
            } => Layout::Vertical {
                spacing,
                padding: padding_val,
                children,
            },
            Layout::Grid {
                columns,
                spacing,
                children,
                ..
            } => Layout::Grid {
                columns,
                spacing,
                padding: padding_val,
                children,
            },
            other => other,
        }
    }

    pub fn add(&mut self, widget: Box<dyn Widget>) {
        match self {
            Layout::Horizontal { children, .. }
            | Layout::Vertical { children, .. }
            | Layout::Grid { children, .. }
            | Layout::Stack { children }
            | Layout::Overlay { children } => {
                children.push(widget);
            }
        }
    }

    pub fn layout_size(&mut self, constraint: LayoutConstraint) -> Size {
        match self {
            Layout::Horizontal {
                spacing,
                padding,
                children,
            } => {
                let spacing = *spacing;
                let padding = *padding;
                let mut width: f32 = padding * 2.0;
                let mut height: f32 = 0.0;
                for child in children.iter_mut() {
                    let child_constraint = LayoutConstraint {
                        min_width: 0.0,
                        max_width: constraint.max_width - width,
                        min_height: 0.0,
                        max_height: constraint.max_height,
                    };
                    let size = child.layout(child_constraint);
                    width += size.width + spacing;
                    height = height.max(size.height);
                }
                if !children.is_empty() {
                    width -= spacing;
                }
                width += padding * 2.0;
                height += padding * 2.0;
                constraint.clamp(Size { width, height })
            }
            Layout::Vertical {
                spacing,
                padding,
                children,
            } => {
                let spacing = *spacing;
                let padding = *padding;
                let mut width: f32 = 0.0;
                let mut height: f32 = padding * 2.0;
                for child in children.iter_mut() {
                    let child_constraint = LayoutConstraint {
                        min_width: 0.0,
                        max_width: constraint.max_width,
                        min_height: 0.0,
                        max_height: constraint.max_height - height,
                    };
                    let size = child.layout(child_constraint);
                    width = width.max(size.width);
                    height += size.height + spacing;
                }
                if !children.is_empty() {
                    height -= spacing;
                }
                height += padding * 2.0;
                constraint.clamp(Size { width, height })
            }
            Layout::Grid {
                columns,
                spacing,
                padding,
                children,
            } => {
                let spacing = *spacing;
                let padding = *padding;
                let columns = *columns;
                let _width: f32 = padding * 2.0;
                let num_rows = children.len().div_ceil(columns);
                let mut heights: Vec<f32> = vec![0.0; num_rows];
                let col_width = if columns > 0 {
                    (constraint.max_width - padding * 2.0 - spacing * (columns as f32 - 1.0))
                        / columns as f32
                } else {
                    0.0
                };
                for (i, child) in children.iter_mut().enumerate() {
                    let row = i / columns;
                    let child_size = child.layout(LayoutConstraint::tight(Size::new(
                        col_width,
                        constraint.max_height,
                    )));
                    heights[row] = heights[row].max(child_size.height);
                }
                let total_height: f32 = heights.iter().sum::<f32>()
                    + spacing * (heights.len().saturating_sub(1) as f32)
                    + padding * 2.0;
                let total_width = if columns > 0 {
                    let cols = children.len().min(columns) as f32;
                    col_width * cols + spacing * (cols - 1.0) + padding * 2.0
                } else {
                    padding * 2.0
                };
                constraint.clamp(Size::new(total_width, total_height))
            }
            Layout::Stack { children } => {
                let mut size = Size::ZERO;
                for child in children.iter_mut() {
                    let child_size = child.layout(constraint);
                    size.width = size.width.max(child_size.width);
                    size.height = size.height.max(child_size.height);
                }
                constraint.clamp(size)
            }
            Layout::Overlay { children } => {
                let mut size = Size::ZERO;
                for child in children.iter_mut() {
                    let child_size = child.layout(constraint);
                    size.width = size.width.max(child_size.width);
                    size.height = size.height.max(child_size.height);
                }
                constraint.clamp(size)
            }
        }
    }

    pub fn arrange(&mut self, rect: Rect) {
        match self {
            Layout::Horizontal {
                spacing,
                padding,
                children,
            } => {
                let spacing = *spacing;
                let padding = *padding;
                let mut x = rect.x + padding;
                for child in children.iter_mut() {
                    let child_height = child.rect().height;
                    child.set_rect(Rect::new(
                        x,
                        rect.y + padding,
                        child.rect().width.max(0.0),
                        child_height,
                    ));
                    x += child.rect().width + spacing;
                }
            }
            Layout::Vertical {
                spacing,
                padding,
                children,
            } => {
                let spacing = *spacing;
                let padding = *padding;
                let mut y = rect.y + padding;
                for child in children.iter_mut() {
                    let child_width = child.rect().width;
                    let child_height = child.rect().height;
                    child.set_rect(Rect::new(rect.x + padding, y, child_width, child_height));
                    y += child_height + spacing;
                }
            }
            Layout::Grid {
                columns,
                spacing,
                padding,
                children,
            } => {
                let spacing = *spacing;
                let padding = *padding;
                let columns = *columns;
                let col_width = if columns > 0 {
                    (rect.width - padding * 2.0 - spacing * (columns as f32 - 1.0)) / columns as f32
                } else {
                    0.0
                };
                let mut x = rect.x + padding;
                let mut y = rect.y + padding;
                for (i, child) in children.iter_mut().enumerate() {
                    if i > 0 && i % columns == 0 {
                        x = rect.x + padding;
                        y += child.rect().height + spacing;
                    }
                    child.set_rect(Rect::new(x, y, col_width.max(0.0), child.rect().height));
                    x += col_width + spacing;
                }
            }
            Layout::Stack { children } => {
                for child in children.iter_mut() {
                    child.set_rect(rect);
                }
            }
            Layout::Overlay { children } => {
                for child in children.iter_mut() {
                    child.set_rect(rect);
                }
            }
        }
    }
}
