use crate::traits::UIAutomationProvider;
use serde_json::{Value, json};
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Accessibility::*;
use windows::core::*;

pub struct UIAutomationBridge {
    automation: IUIAutomation3,
    cache_request: IUIAutomationCacheRequest,
}

unsafe impl Send for UIAutomationBridge {}
unsafe impl Sync for UIAutomationBridge {}

impl UIAutomationProvider for UIAutomationBridge {
    fn get_root_json(&self) -> anyhow::Result<Value> {
        let root = self
            .get_root()
            .map_err(|e| anyhow::anyhow!("Failed to get root: {}", e))?;
        Ok(self.traverse_tree_native(&root, 0))
    }

    fn get_focused_json(&self) -> anyhow::Result<Value> {
        let element = self
            .get_focused_element()
            .map_err(|e| anyhow::anyhow!("Failed to get focused: {}", e))?;
        Ok(self.traverse_tree_native(&element, 5))
    }

    fn find_element(&self, query: &str) -> anyhow::Result<Option<RECT>> {
        self.find_element_native(query)
            .map_err(|e| anyhow::anyhow!("Search failed: {}", e))
    }
}

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
            self.automation
                .GetFocusedElementBuildCache(&self.cache_request)
        }
    }

    pub fn get_root(&self) -> Result<IUIAutomationElement> {
        unsafe {
            self.automation
                .GetRootElementBuildCache(&self.cache_request)
        }
    }

    fn traverse_tree_native(&self, element: &IUIAutomationElement, depth: u32) -> Value {
        if depth > 5 {
            return json!({ "note": "max depth reached" });
        }

        unsafe {
            let name = element
                .CachedName()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let automation_id = element
                .CachedAutomationId()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let class_name = element
                .CachedClassName()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let control_type = element
                .CachedControlType()
                .unwrap_or(UIA_CONTROLTYPE_ID(0))
                .0;
            let rect = element.CachedBoundingRectangle().unwrap_or_default();

            let mut children_list = Vec::new();
            if let Ok(children) = self.get_children_native(element) {
                let count = children.Length().unwrap_or(0);
                for i in 0..count {
                    if let Ok(child) = children.GetElement(i) {
                        children_list.push(self.traverse_tree_native(&child, depth + 1));
                    }
                }
            }

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
    }

    fn get_children_native(
        &self,
        element: &IUIAutomationElement,
    ) -> Result<IUIAutomationElementArray> {
        unsafe {
            let condition = self.automation.CreateTrueCondition()?;
            element.FindAllBuildCache(TreeScope_Children, &condition, &self.cache_request)
        }
    }

    fn find_element_native(&self, query: &str) -> Result<Option<RECT>> {
        unsafe {
            let root = self.automation.GetRootElement()?;
            let var_name: VARIANT = query.into();
            let cond_name = self
                .automation
                .CreatePropertyCondition(UIA_NamePropertyId, &var_name)?;
            let var_id: VARIANT = query.into();
            let cond_id = self
                .automation
                .CreatePropertyCondition(UIA_AutomationIdPropertyId, &var_id)?;
            let condition = self.automation.CreateOrCondition(&cond_name, &cond_id)?;

            if let Ok(element) =
                root.FindFirstBuildCache(TreeScope_Descendants, &condition, &self.cache_request)
            {
                return Ok(Some(element.CachedBoundingRectangle()?));
            }
            Ok(None)
        }
    }
}
