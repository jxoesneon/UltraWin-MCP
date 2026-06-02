use windows::core::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::Foundation::*;
use serde_json::{json, Value};

/// Trait to abstract UI Automation elements for testing.
pub trait TUIElement {
    fn cached_name(&self) -> String;
    fn cached_automation_id(&self) -> String;
    fn cached_class_name(&self) -> String;
    fn cached_control_type(&self) -> i32;
    fn cached_bounding_rectangle(&self) -> RECT;
    fn get_children(&self) -> Vec<Box<dyn TUIElement>>;
}

/// Implementation of TUIElement for real Windows UI Automation elements.
pub struct RealUIElement {
    pub element: IUIAutomationElement,
    pub automation: IUIAutomation3,
    pub cache_request: IUIAutomationCacheRequest,
}

impl TUIElement for RealUIElement {
    fn cached_name(&self) -> String {
        unsafe { self.element.CachedName().map(|s| s.to_string()).unwrap_or_default() }
    }
    fn cached_automation_id(&self) -> String {
        unsafe { self.element.CachedAutomationId().map(|s| s.to_string()).unwrap_or_default() }
    }
    fn cached_class_name(&self) -> String {
        unsafe { self.element.CachedClassName().map(|s| s.to_string()).unwrap_or_default() }
    }
    fn cached_control_type(&self) -> i32 {
        unsafe { self.element.CachedControlType().unwrap_or(UIA_CONTROLTYPE_ID(0)).0 }
    }
    fn cached_bounding_rectangle(&self) -> RECT {
        unsafe { self.element.CachedBoundingRectangle().unwrap_or_default() }
    }
    fn get_children(&self) -> Vec<Box<dyn TUIElement>> {
        let mut children_list = Vec::new();
        unsafe {
            if let Ok(condition) = self.automation.CreateTrueCondition() {
                if let Ok(children) = self.element.FindAllBuildCache(TreeScope_Children, &condition, &self.cache_request) {
                    let count = children.Length().unwrap_or(0);
                    for i in 0..count {
                        if let Ok(child) = children.GetElement(i) {
                            children_list.push(Box::new(RealUIElement {
                                element: child,
                                automation: self.automation.clone(),
                                cache_request: self.cache_request.clone(),
                            }) as Box<dyn TUIElement>);
                        }
                    }
                }
            }
        }
        children_list
    }
}

pub struct UIAutomationBridge {
    automation: IUIAutomation3,
    cache_request: IUIAutomationCacheRequest,
}

// SAFETY: COM interfaces are thread-safe when initialized with COINIT_MULTITHREADED.
unsafe impl Send for UIAutomationBridge {}
unsafe impl Sync for UIAutomationBridge {}

impl UIAutomationBridge {
    pub fn new() -> Result<Self> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let automation: IUIAutomation3 = CoCreateInstance(&CUIAutomation8, None, CLSCTX_ALL)?;
            let cache_request = automation.CreateCacheRequest()?;
            cache_request.AddProperty(UIA_NamePropertyId)?;
            cache_request.AddProperty(UIA_AutomationIdPropertyId)?;
            cache_request.AddProperty(UIA_BoundingRectanglePropertyId)?;
            cache_request.AddProperty(UIA_ControlTypePropertyId)?;
            cache_request.AddProperty(UIA_ClassNamePropertyId)?;
            cache_request.SetTreeScope(TreeScope_Element)?;

            Ok(Self {
                automation,
                cache_request,
            })
        }
    }

    pub fn get_focused_element(&self) -> Result<IUIAutomationElement> {
        unsafe {
            self.automation.GetFocusedElementBuildCache(&self.cache_request)
        }
    }

    pub fn find_element_by_name(&self, name: &str) -> Result<IUIAutomationElement> {
        unsafe {
            let root = self.automation.GetRootElement()?;
            let var: VARIANT = name.into();
            let condition = self.automation.CreatePropertyCondition(UIA_NamePropertyId, &var)?;
            root.FindFirstBuildCache(TreeScope_Descendants, &condition, &self.cache_request)
        }
    }

    pub fn get_children(&self, element: &IUIAutomationElement) -> Result<IUIAutomationElementArray> {
        unsafe {
            let condition = self.automation.CreateTrueCondition()?;
            element.FindAllBuildCache(TreeScope_Children, &condition, &self.cache_request)
        }
    }

    pub fn get_root(&self) -> Result<IUIAutomationElement> {
        unsafe {
            self.automation.GetRootElementBuildCache(&self.cache_request)
        }
    }

    pub fn traverse_tree(&self, element: &IUIAutomationElement, depth: u32) -> Value {
        let real_el = RealUIElement {
            element: element.clone(),
            automation: self.automation.clone(),
            cache_request: self.cache_request.clone(),
        };
        traverse_element(&real_el, depth)
    }

    pub fn find_element(&self, query: &str) -> Result<Option<RECT>> {
        unsafe {
            let root = self.automation.GetRootElement()?;

            // Try by name first
            let var_name: VARIANT = query.into();
            let cond_name = self.automation.CreatePropertyCondition(UIA_NamePropertyId, &var_name)?;

            // Try by automation id
            let var_id: VARIANT = query.into();
            let cond_id = self.automation.CreatePropertyCondition(UIA_AutomationIdPropertyId, &var_id)?;

            let condition = self.automation.CreateOrCondition(&cond_name, &cond_id)?;

            if let Ok(element) = root.FindFirstBuildCache(TreeScope_Descendants, &condition, &self.cache_request) {
                return Ok(Some(element.CachedBoundingRectangle()?));
            }

            Ok(None)
        }
    }
}

pub fn traverse_element(element: &dyn TUIElement, depth: u32) -> Value {
    if depth > 5 {
        return json!({ "note": "max depth reached" });
    }

    let name = element.cached_name();
    let automation_id = element.cached_automation_id();
    let class_name = element.cached_class_name();
    let control_type = element.cached_control_type();
    let rect = element.cached_bounding_rectangle();

    let children = element.get_children();
    let children_list: Vec<Value> = children
        .into_iter()
        .map(|child| traverse_element(child.as_ref(), depth + 1))
        .collect();

    json!({
        "name": name,
        "automation_id": automation_id,
        "class_name": class_name,
        "control_type": control_type,
        "bounding_box": {
            "left": rect.left,
            "top": rect.top,
            "right": rect.right,
            "bottom": rect.bottom
        },
        "children": children_list
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockUIElement {
        name: String,
        automation_id: String,
        class_name: String,
        control_type: i32,
        rect: RECT,
        children: Vec<MockUIElement>,
    }

    impl TUIElement for MockUIElement {
        fn cached_name(&self) -> String { self.name.clone() }
        fn cached_automation_id(&self) -> String { self.automation_id.clone() }
        fn cached_class_name(&self) -> String { self.class_name.clone() }
        fn cached_control_type(&self) -> i32 { self.control_type }
        fn cached_bounding_rectangle(&self) -> RECT { self.rect }
        fn get_children(&self) -> Vec<Box<dyn TUIElement>> {
            self.children.iter().map(|c| {
                Box::new(c.clone()) as Box<dyn TUIElement>
            }).collect()
        }
    }

    #[test]
    fn test_traverse_tree_recursive() {
        let mock_root = MockUIElement {
            name: "Root".into(),
            automation_id: "root_id".into(),
            class_name: "Window".into(),
            control_type: 50000,
            rect: RECT { left: 0, top: 0, right: 100, bottom: 100 },
            children: vec![
                MockUIElement {
                    name: "Button".into(),
                    automation_id: "btn_1".into(),
                    class_name: "Button".into(),
                    control_type: 50001,
                    rect: RECT { left: 10, top: 10, right: 50, bottom: 30 },
                    children: vec![],
                }
            ],
        };

        let result = traverse_element(&mock_root, 0);
        
        assert_eq!(result["name"], "Root");
        assert_eq!(result["children"].as_array().unwrap().len(), 1);
        assert_eq!(result["children"][0]["name"], "Button");
    }

    #[test]
    fn test_traverse_tree_depth_limit() {
        fn create_deep_tree(depth: u32) -> MockUIElement {
            if depth == 0 {
                MockUIElement {
                    name: "Leaf".into(),
                    automation_id: "".into(),
                    class_name: "".into(),
                    control_type: 0,
                    rect: RECT::default(),
                    children: vec![],
                }
            } else {
                MockUIElement {
                    name: format!("Depth {}", depth),
                    automation_id: "".into(),
                    class_name: "".into(),
                    control_type: 0,
                    rect: RECT::default(),
                    children: vec![create_deep_tree(depth - 1)],
                }
            }
        }

        let deep_root = create_deep_tree(10);
        let result = traverse_element(&deep_root, 0);
        
        // Find the "max depth reached" note
        let mut current = &result;
        let mut found_note = false;
        for _ in 0..10 {
            if let Some(note) = current.get("note") {
                if note == "max depth reached" {
                    found_note = true;
                    break;
                }
            }
            if let Some(children) = current.get("children") {
                if let Some(first_child) = children.as_array().and_then(|a| a.get(0)) {
                    current = first_child;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        assert!(found_note);
    }
}
