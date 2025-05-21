import React, { useState } from "react";
import { useTheme } from "@shared/lib/theming";
import { Settings, Sun, Moon, Bell, BellOff, Volume2, VolumeX, Sliders } from "lucide-react";
import { CollapsibleSidebar } from "./collapsible-sidebar";

interface SettingsSidebarProps {
  children?: React.ReactNode;
}

export const SettingsSidebar: React.FC<SettingsSidebarProps> = ({ children }) => {
  // 控制折叠状态
  const [collapsed, setCollapsed] = useState<boolean>(true);
  
  // 使用新的useTheme钩子
  const { mode, toggleTheme } = useTheme();

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
    if (mode === 'light') {
      return {
        icon: <Sun 
          className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
          onClick={toggleTheme}
        />,
        tooltip: '切换到暗色模式'
      };
    } else {
      return {
        icon: <Moon 
          className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
          onClick={toggleTheme}
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
        <div className="flex flex-col space-y-4">
          {/* 设置图标 */}
          <div className="relative group">
            <Settings className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 transition-colors duration-300" />
            <span className="absolute left-full ml-2 px-3 py-2 bg-white/90 dark:bg-slate-800/90 text-base font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              设置
            </span>
          </div>
          
          {/* 主题切换图标 */}
          <div className="relative group">
            {themeConfig.icon}
            <span className="absolute left-full ml-2 px-3 py-2 bg-white/90 dark:bg-slate-800/90 text-base font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              {themeConfig.tooltip}
            </span>
          </div>
          
          {/* 通知设置 */}
          <div className="relative group">
            {notifications ? (
              <Bell 
                className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleNotifications}
              />
            ) : (
              <BellOff 
                className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleNotifications}
              />
            )}
            <span className="absolute left-full ml-2 px-3 py-2 bg-white/90 dark:bg-slate-800/90 text-base font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              {notifications ? '关闭通知' : '开启通知'}
            </span>
          </div>
          
          {/* 声音设置 */}
          <div className="relative group">
            {soundEnabled ? (
              <Volume2 
                className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleSound}
              />
            ) : (
              <VolumeX 
                className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
                onClick={toggleSound}
              />
            )}
            <span className="absolute left-full ml-2 px-3 py-2 bg-white/90 dark:bg-slate-800/90 text-base font-medium text-gray-700 dark:text-gray-100 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              {soundEnabled ? '关闭声音' : '开启声音'}
            </span>
          </div>
          
          {/* 动画速度设置 */}
          <div className="relative group">
            <Sliders 
              className="w-6 h-6 text-gray-500 dark:text-gray-300 group-hover:text-gray-700 dark:group-hover:text-gray-100 cursor-pointer transition-colors duration-300"
            />
            <div className="absolute left-full ml-2 p-3 bg-white/90 dark:bg-slate-800/90 rounded shadow-md dark:shadow-slate-900/30 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300 whitespace-nowrap border border-gray-200 dark:border-slate-700/50">
              <div className="flex flex-col space-y-2">
                <span className="text-base font-medium text-gray-700 dark:text-gray-100">动画速度: {animationSpeed}x</span>
                <input
                  type="range"
                  min="0.5"
                  max="2"
                  step="0.1"
                  value={animationSpeed}
                  onChange={handleAnimationSpeedChange}
                  className="w-40 h-2.5 bg-gray-200 dark:bg-slate-700 rounded-lg appearance-none cursor-pointer"
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