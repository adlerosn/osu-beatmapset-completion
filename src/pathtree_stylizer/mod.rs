use std::collections::BTreeMap as Map;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone, new)]
pub struct PathTreeStylized<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug,
{
    segment: String,
    data: Option<T>,
    colors: Option<Map<T, (String, String)>>,
    base_color: Option<(String, String)>,
    children: Vec<PathTreeStylized<T>>,
}

impl<T> PathTreeStylized<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug,
{
    fn get_child_mut(&mut self, segment: &String) -> Option<&mut PathTreeStylized<T>> {
        self.children
            .iter_mut()
            .filter(|x| x.segment == *segment)
            .next()
    }
    fn add_child(&mut self, path: &mut dyn Iterator<Item = String>, data: T) {
        if let Some(segment) = path.next() {
            let child_opt = self.get_child_mut(&segment);
            if let Some(child) = child_opt {
                child.add_child(path, data);
            } else {
                let mut child = PathTreeStylized::new(
                    segment.clone(),
                    None,
                    self.colors.clone(),
                    self.base_color.clone(),
                    vec![],
                );
                child.add_child(path, data);
                self.children.push(child);
            }
        } else {
            self.data = Some(data);
        }
    }

    pub fn fill_data_greatest(&mut self) {
        for child in self.children.iter_mut() {
            child.fill_data_greatest();
        }
        if self.data == None {
            let mut children_values: Vec<&T> = self
                .children
                .iter()
                .filter_map(|x| x.data.as_ref())
                .collect();
            children_values.sort();
            self.data = children_values.last().and_then(|x| Some((*x).clone()));
        }
    }

    pub fn sort(&mut self) {
        for child in self.children.iter_mut() {
            child.sort();
        }
        self.children.sort();
    }

    pub fn reverse(&mut self) {
        for child in self.children.iter_mut() {
            child.reverse();
        }
        self.children.reverse();
    }

    pub fn set_colors(
        &mut self,
        colors: Option<Map<T, (String, String)>>,
        base_color: Option<(String, String)>,
    ) {
        for child in self.children.iter_mut() {
            child.set_colors(colors.clone(), base_color.clone());
        }
        self.colors = colors;
        self.base_color = base_color;
    }

    fn printable_string_(&self, parents: Vec<(bool, &PathTreeStylized<T>)>) -> String {
        let mut sb = "".to_string();
        let current: &PathTreeStylized<T> = parents.last().unwrap().1;
        let (base_color_start, base_color_end) = current
            .base_color
            .clone()
            .unwrap_or(("".to_string(), "".to_string()));
        sb.push_str(&base_color_start);
        for (seq, (_, parent)) in parents.iter().enumerate() {
            if seq + 1 != parents.len() {
                let parent_is_last = parents[seq + 1].0;
                sb.push_str(&format!("{} ", " ".repeat(parent.segment.len() + 2)));
                if parent_is_last {
                    if seq + 2 >= parents.len() {
                        sb.push('`');
                    } else {
                        sb.push(' ');
                    }
                } else {
                    sb.push('|');
                }
            }
        }
        sb.push_str(&format!("- {}", current.segment));
        sb.push_str(" +=> ");
        sb.push_str(&base_color_end);
        if let Some(data) = &current.data {
            let color: (String, String) = current
                .colors
                .clone()
                .unwrap_or(Map::new())
                .get(&data)
                .and_then(|x| Some(x.clone()))
                .unwrap_or(("".to_string(), "".to_string()));
            sb.push_str(&format!("{}{:?}{}", color.0, &data, color.1));
        }
        sb.push_str("\n");
        for (seq, child) in current.children.iter().enumerate() {
            let is_last = seq + 1 == current.children.len();
            let mut parent_with_me = parents.clone();
            parent_with_me.push((is_last, &child));
            sb.push_str(&child.printable_string_(parent_with_me));
        }
        return sb;
    }

    fn printable_string(&self) -> String {
        self.printable_string_(vec![(true, self)])
    }
}

impl<T> From<&Vec<(PathBuf, T)>> for PathTreeStylized<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug,
{
    fn from(v: &Vec<(PathBuf, T)>) -> Self {
        let mut root_tree = Self::new("".to_string(), None, None, None, vec![]);
        for (path, data) in v.iter() {
            root_tree.add_child(
                &mut path.iter().map(|x| x.to_str().unwrap().to_string()),
                (*data).clone(),
            );
        }
        root_tree
    }
}

impl<T> std::fmt::Display for PathTreeStylized<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.printable_string())
    }
}

impl<T> PartialOrd for PathTreeStylized<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(match self.data.cmp(&other.data) {
            std::cmp::Ordering::Equal => self.segment.cmp(&other.segment),
            other => other,
        });
    }
}

impl<T> Ord for PathTreeStylized<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
