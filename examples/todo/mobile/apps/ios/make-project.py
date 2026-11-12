#!/usr/bin/env python3
"""Generate the small dependency-free Xcode project (checked in beside this script)."""
from pathlib import Path
import hashlib

def ident(name): return hashlib.sha1(name.encode()).hexdigest()[:24].upper()
objects={}
def add(name, value): objects[ident(name)]=value; return ident(name)
def ref(name): return ident(name)
source_files=['Sources/TodoUI/TodoStore.swift','Sources/TodoUI/TodoViews.swift','Sources/TodoApp/TodoApp.swift']
for path in source_files+['Sources/TodoUI/seeds.json']:
    add(path, f'{{isa = PBXFileReference; lastKnownFileType = {"sourcecode.swift" if path.endswith("swift") else "text.json"}; path = "{path}"; sourceTree = "<group>"; }}')
for path in source_files[:2]+['Sources/TodoUI/seeds.json']:
    add('build:'+path, f'{{isa = PBXBuildFile; fileRef = {ref(path)}; }}')
add('appmain',f'{{isa = PBXBuildFile; fileRef = {ref(source_files[2])}; }}')
add('framework', '{isa = PBXFileReference; explicitFileType = wrapper.framework; path = TodoUI.framework; sourceTree = BUILT_PRODUCTS_DIR; }')
add('appproduct','{isa = PBXFileReference; explicitFileType = wrapper.application; path = Todo.app; sourceTree = BUILT_PRODUCTS_DIR; }')
add('linkframework',f'{{isa = PBXBuildFile; fileRef = {ref("framework")}; }}')
add('embedframework',f'{{isa = PBXBuildFile; fileRef = {ref("framework")}; settings = {{ATTRIBUTES = (CodeSignOnCopy, RemoveHeadersOnCopy); }}; }}')
add('sources-ui',f'{{isa = PBXSourcesBuildPhase; buildActionMask = 2147483647; files = ({ref("build:"+source_files[0])}, {ref("build:"+source_files[1])}); runOnlyForDeploymentPostprocessing = 0; }}')
add('resources-ui',f'{{isa = PBXResourcesBuildPhase; buildActionMask = 2147483647; files = ({ref("build:Sources/TodoUI/seeds.json")}); runOnlyForDeploymentPostprocessing = 0; }}')
add('sources-app',f'{{isa = PBXSourcesBuildPhase; buildActionMask = 2147483647; files = ({ref("appmain")}); runOnlyForDeploymentPostprocessing = 0; }}')
add('link-app',f'{{isa = PBXFrameworksBuildPhase; buildActionMask = 2147483647; files = ({ref("linkframework")}); runOnlyForDeploymentPostprocessing = 0; }}')
add('embed-app',f'{{isa = PBXCopyFilesBuildPhase; buildActionMask = 2147483647; dstPath = ""; dstSubfolderSpec = 10; files = ({ref("embedframework")}); name = "Embed Frameworks"; runOnlyForDeploymentPostprocessing = 0; }}')
add('dependency',f'{{isa = PBXTargetDependency; target = {ref("target-ui")}; }}')
for target in ['project','ui','app']:
    for config in ['Debug','Release']:
        settings='ALWAYS_SEARCH_USER_PATHS = NO; SWIFT_VERSION = 5.0; IPHONEOS_DEPLOYMENT_TARGET = 26.0; SDKROOT = iphoneos; SUPPORTED_PLATFORMS = "iphoneos iphonesimulator"; TARGETED_DEVICE_FAMILY = "1,2"; CODE_SIGN_STYLE = Automatic; '
        if target=='project': settings += 'SWIFT_OPTIMIZATION_LEVEL = "'+('-Onone' if config=='Debug' else '-O')+'"; '
        else:
            settings+='GENERATE_INFOPLIST_FILE = YES; PRODUCT_NAME = "$(TARGET_NAME)"; '
            settings+='PRODUCT_BUNDLE_IDENTIFIER = org.omnicui.todo'+('.ui; DEFINES_MODULE = YES; SKIP_INSTALL = YES; ' if target=='ui' else '; INFOPLIST_KEY_CFBundleDisplayName = Tasks; INFOPLIST_KEY_UILaunchScreen_Generation = YES; INFOPLIST_KEY_UIApplicationSceneManifest_Generation = YES; LD_RUNPATH_SEARCH_PATHS = "$(inherited) @executable_path/Frameworks"; ')
        add(target+config,f'{{isa = XCBuildConfiguration; buildSettings = {{{settings}}}; name = {config}; }}')
    add(target+'configs',f'{{isa = XCConfigurationList; buildConfigurations = ({ref(target+"Debug")}, {ref(target+"Release")}); defaultConfigurationIsVisible = 0; defaultConfigurationName = Release; }}')
add('target-ui',f'{{isa = PBXNativeTarget; buildConfigurationList = {ref("uiconfigs")}; buildPhases = ({ref("sources-ui")}, {ref("resources-ui")}); buildRules = (); dependencies = (); name = TodoUI; productName = TodoUI; productReference = {ref("framework")}; productType = "com.apple.product-type.framework"; }}')
add('target-app',f'{{isa = PBXNativeTarget; buildConfigurationList = {ref("appconfigs")}; buildPhases = ({ref("sources-app")}, {ref("link-app")}, {ref("embed-app")}); buildRules = (); dependencies = ({ref("dependency")}); name = Todo; productName = Todo; productReference = {ref("appproduct")}; productType = "com.apple.product-type.application"; }}')
add('products',f'{{isa = PBXGroup; children = ({ref("framework")}, {ref("appproduct")}); name = Products; sourceTree = "<group>"; }}')
add('main', '{isa = PBXGroup; children = ('+', '.join(ref(p) for p in source_files+['Sources/TodoUI/seeds.json'])+f', {ref("products")}); sourceTree = "<group>"; }}')
add('project',f'{{isa = PBXProject; attributes = {{LastUpgradeCheck = 2700; }}; buildConfigurationList = {ref("projectconfigs")}; compatibilityVersion = "Xcode 14.0"; developmentRegion = en; hasScannedForEncodings = 0; knownRegions = (en, Base); mainGroup = {ref("main")}; productRefGroup = {ref("products")}; projectDirPath = ""; projectRoot = ""; targets = ({ref("target-app")}, {ref("target-ui")}); }}')
p=Path(__file__).parent/'Todo.xcodeproj'
p.mkdir(exist_ok=True)
(p/'project.pbxproj').write_text('// !$*UTF8*$!\n{ archiveVersion = 1; classes = {}; objectVersion = 56; objects = {\n'+ '\n'.join(f'{k} = {v};' for k,v in objects.items())+f'\n}}; rootObject = {ref("project")}; }}\n')
