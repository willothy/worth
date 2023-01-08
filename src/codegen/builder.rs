use crate::{asm, asm_line, label};

#[derive(Debug, Clone)]
pub struct Builder {
    bss: Segment,
    text: Segment,
    data: Segment,
    pub insert_segment: SegmentKind,
    pub insert_point: InsertPoint,
    const_str_counter: usize,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub lines: Vec<String>,
    pub has_header: bool,
}

impl Segment {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            has_header: false,
        }
    }
    pub fn insert(&mut self, idx: usize, line: String) {
        self.lines.insert(idx, line);
    }

    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }

    pub fn join(&self, sep: &str) -> String {
        self.lines.join(sep)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentKind {
    Bss,
    Text,
    Data,
}

impl Builder {
    pub fn new() -> Self {
        let tmp = Self {
            bss: Segment::new(),
            text: Segment::new(),
            data: Segment::new(),
            insert_segment: SegmentKind::Bss,
            insert_point: InsertPoint::End,
            const_str_counter: 0,
        };
        tmp
    }

    pub fn set_insert_segment(&mut self, segment: SegmentKind) {
        self.insert_segment = segment;
        self.insert_point = InsertPoint::End;
    }

    pub fn get_insert_segment(&self) -> &Segment {
        match self.insert_segment {
            SegmentKind::Bss => &self.bss,
            SegmentKind::Text => &self.text,
            SegmentKind::Data => &self.data,
        }
    }

    pub fn set_insert_point(&mut self, ins_pt: InsertPoint) {
        self.insert_point = ins_pt;
    }

    pub fn insert(&mut self, line: String) {
        match self.insert_point {
            InsertPoint::Start => match self.insert_segment {
                SegmentKind::Bss => self.bss.insert(0, line),
                SegmentKind::Text => self.text.insert(0, line),
                SegmentKind::Data => self.data.insert(0, line),
            },
            InsertPoint::End => match self.insert_segment {
                SegmentKind::Bss => self.bss.push(line),
                SegmentKind::Text => self.text.push(line),
                SegmentKind::Data => self.data.push(line),
            },
            InsertPoint::Line(line_no) => match self.insert_segment {
                SegmentKind::Bss => self.bss.insert(line_no, line),
                SegmentKind::Text => self.text.insert(line_no, line),
                SegmentKind::Data => self.data.insert(line_no, line),
            },
        }
    }

    pub fn new_const_str(&mut self, value: &str) -> usize {
        let prev_ins_pt = self.insert_point;
        let prev_ins_seg = self.insert_segment;
        self.set_insert_segment(SegmentKind::Data);
        self.set_insert_point(InsertPoint::End);
        let label = format!("const_str_{}", self.const_str_counter);
        label!(self, "{}", label);
        let bytes_str = value
            .as_bytes()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        asm!(self, ("db", "{}", bytes_str));
        self.const_str_counter += 1;
        self.set_insert_segment(prev_ins_seg);
        self.set_insert_point(prev_ins_pt);
        self.const_str_counter - 1
    }

    pub fn count_lines(&self) -> usize {
        // + 3 for segment headers
        self.bss.lines.len() + self.text.lines.len() + self.data.lines.len() + 3
    }

    pub fn finalize(self) -> String {
        let mut output = String::new();
        output += "segment .bss\n";
        output += &self.bss.join("\n");
        output += "\n\n";
        output += "segment .text\n";
        output += &self.text.join("\n");
        output += "\n\n";
        output += "segment .data\n";
        output += &self.data.join("\n");
        output += "\n\n";
        output
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InsertPoint {
    Start,
    End,
    Line(usize),
}
