use windows::core::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Accessibility::*;

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
}
