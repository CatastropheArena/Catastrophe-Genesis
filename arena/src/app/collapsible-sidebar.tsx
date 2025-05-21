import React, { useState, useEffect } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';

interface CollapsibleSidebarProps {
  /**
   * 是否折叠侧边栏
   * @default true
   */
  collapsed?: boolean;
  
  /**
   * 折叠状态改变时的回调函数
   */
  onCollapsedChange?: (collapsed: boolean) => void;
  
  /**
   * 侧边栏内容
   */
  children?: React.ReactNode;
}

export const CollapsibleSidebar: React.FC<CollapsibleSidebarProps> = ({
  collapsed: propCollapsed = true,
  onCollapsedChange,
  children,
}) => {
  // 控制折叠状态
  const [isCollapsed, setIsCollapsed] = useState<boolean>(propCollapsed);
  
  // 监听 props 中折叠状态的变化
  useEffect(() => {
    setIsCollapsed(propCollapsed);
  }, [propCollapsed]);
  
  // 控制悬浮状态
  const [isHovered, setIsHovered] = useState<boolean>(false);
  
  // 切换折叠状态
  const toggleSidebar = () => {
    const newCollapsedState = !isCollapsed;
    setIsCollapsed(newCollapsedState);
    if (onCollapsedChange) {
      onCollapsedChange(newCollapsedState);
    }
  };
  
  // 处理鼠标事件
  const handleMouseEnter = () => {
    setIsHovered(true);
  };
  
  const handleMouseLeave = () => {
    setIsHovered(false);
  };
  
  return (
    <div
      className="absolute flex h-screen z-[2147483645]"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* 侧边栏内容 */}
      <div
        className={`
          flex flex-col items-center p-4 
          bg-white/80 dark:bg-slate-900/80 backdrop-blur-md
          shadow-lg dark:shadow-slate-900/20 rounded-r-xl h-fit self-center 
          relative transition-all duration-300 ease-in-out
          border-r border-t border-b border-gray-200 dark:border-slate-700/50
          ${isCollapsed ? 'hidden' : 'w-12'}
        `}
      >
        {children}
      </div>
      
      {/* 切换按钮 - 只有在折叠状态或鼠标悬浮时显示 */}
      {(isCollapsed || isHovered) && (
        <div 
          className="absolute left-full top-1/2 -translate-y-1/2 transform"
        >
            {isCollapsed ? (
              <ChevronRight 
                className="w-8 h-8 transition-transform duration-300 ease-in-out hover:scale-110 text-gray-500 dark:text-gray-300 hover:text-violet-600 dark:hover:text-violet-400 cursor-pointer"
                onClick={toggleSidebar}
              />
            ) : (
              <ChevronLeft 
                className="w-8 h-8 transition-transform duration-300 ease-in-out hover:scale-110 text-gray-500 dark:text-gray-300 hover:text-violet-600 dark:hover:text-violet-400 cursor-pointer"
                onClick={toggleSidebar}
              />
            )}
        </div>
      )}
    </div>
  );
}; 