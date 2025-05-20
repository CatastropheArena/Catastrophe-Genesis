import React, { useState } from "react";
import { useTheme } from "@shared/lib/theming";
import { Settings, Sun, Moon, Bell, BellOff, Volume2, VolumeX, Sliders, Monitor } from "lucide-react";
import { CollapsibleSidebar } from "./collapsible-sidebar";

interface SettingsSidebarProps {
  children?: React.ReactNode;
}

export const SettingsSidebar: React.FC<SettingsSidebarProps> = ({ children }) => {
  // 控制折叠状态
  const [collapsed, setCollapsed] = useState<boolean>(true);
  
  // 使用新的useTheme钩子
  const { mode, toggleTheme, setTheme } = useTheme();

  // 设置选项
  const [notifications, setNotifications] = useState<boolean>(true);
  const [soundEnabled, setSoundEnabled] = useState<boolean>(true);
  const [animationSpeed, setAnimationSpeed] = useState<number>(1);

  const toggleNotifications = () => setNotifications(!notifications);
  const toggleSound = () => setSoundEnabled(!soundEnabled);
  const handleAnimationSpeedChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setAnimationSpeed(parseFloat(e.target.value));
  };

  // 处理折叠状态变化
  const handleCollapsedChange = (isCollapsed: boolean) => {
    setCollapsed(isCollapsed);
  };

  // 获取主题图标和提示文本
  const getThemeIcon = () => {
    switch (mode) {
      case 'light':
        return {
          icon: <Sun 
            className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
            onClick={() => setTheme('dark')}
          />,
          tooltip: '切换到暗色模式'
        };
      case 'dark':
        return {
          icon: <Moon 
            className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
            onClick={() => setTheme('system')}
          />,
          tooltip: '切换到系统主题'
        };
      case 'system':
      default:
        return {
          icon: <Monitor 
            className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
            onClick={() => setTheme('light')}
          />,
          tooltip: '切换到亮色模式'
        };
    }
  };

  const themeConfig = getThemeIcon();

  return (
    <>
      <CollapsibleSidebar 
        collapsed={collapsed}
        onCollapsedChange={handleCollapsedChange}
      >
        <div className="flex flex-col space-y-2">
          {/* 设置图标 */}
          <div className="relative group">
            <Settings className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 transition-colors duration-300" />
            <span className="absolute left-full ml-2 px-2 py-1 bg-white/90 dark:bg-slate-800/90 text-xs font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              设置
            </span>
          </div>
          
          {/* 主题切换图标 */}
          <div className="relative group">
            {themeConfig.icon}
            <span className="absolute left-full ml-2 px-2 py-1 bg-white/90 dark:bg-slate-800/90 text-xs font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              {themeConfig.tooltip}
            </span>
          </div>
          
          {/* 通知设置 */}
          <div className="relative group">
            {notifications ? (
              <Bell 
                className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleNotifications}
              />
            ) : (
              <BellOff 
                className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleNotifications}
              />
            )}
            <span className="absolute left-full ml-2 px-2 py-1 bg-white/90 dark:bg-slate-800/90 text-xs font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              {notifications ? '关闭通知' : '开启通知'}
            </span>
          </div>
          
          {/* 声音设置 */}
          <div className="relative group">
            {soundEnabled ? (
              <Volume2 
                className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleSound}
              />
            ) : (
              <VolumeX 
                className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleSound}
              />
            )}
            <span className="absolute left-full ml-2 px-2 py-1 bg-white/90 dark:bg-slate-800/90 text-xs font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              {soundEnabled ? '关闭声音' : '开启声音'}
            </span>
          </div>
          
          {/* 动画速度设置 */}
          <div className="relative group">
            <Sliders 
              className="w-3 h-3 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
            />
            <div className="absolute left-full ml-2 p-2 bg-white/90 dark:bg-slate-800/90 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              <div className="flex flex-col space-y-1">
                <span className="text-xs font-medium text-gray-700 dark:text-gray-100">动画速度: {animationSpeed}x</span>
                <input
                  type="range"
                  min="0.5"
                  max="2"
                  step="0.1"
                  value={animationSpeed}
                  onChange={handleAnimationSpeedChange}
                  className="w-24 h-1.5 bg-gray-200 dark:bg-slate-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
            </div>
          </div>
        </div>
      </CollapsibleSidebar>
      
      {/* 将children渲染在侧边栏外部 */}
      {children}
    </>
  );
}; 