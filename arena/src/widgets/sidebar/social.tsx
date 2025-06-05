import React from "react";
import { useSelector } from "react-redux";
import { Link, useNavigate } from "react-router-dom";
import { Cat, PawPrint, UserX, LogOut } from "lucide-react";
import { Button, Menu, MenuItem, IconButton } from "@mui/material";
import { useDispatch } from "@app/store";
import { useAuthStore } from 'src/components/auth';
import { socket } from "@shared/lib/ws";
import { Nullable } from "@shared/lib/typings";
import { viewerModel } from "@entities/viewer";
import { userModel } from "@entities/user";
import { Avatar } from "@shared/ui/atoms";
import { UserWithInterim } from "@shared/api/common";
import { authApi } from "@shared/api/auth";

export const SocialSidebar: React.FC = () => {
  const credentials = viewerModel.useCredentials();
  const interims = userModel.useInterims();
  const friends = useSelector(viewerModel.selectors.friends);
  const hasFriends = friends.data && friends.data.length !== 0;
  const friendsWithInterim = friends.data?.map((friend) => ({
    ...friend,
    interim: interims[friend.id],
  })) as UserWithInterim[];

  // 退出登录相关hook
  const dispatch = useDispatch();
  const { setToken } = useAuthStore();
  const navigate = useNavigate();

  // 控制菜单开关的状态
  const [anchorEl, setAnchorEl] = React.useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  // 点击头像，打开菜单
  const handleProfileClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };
  // 关闭菜单
  const handleMenuClose = () => {
    setAnchorEl(null);
  };

  // 退出登录逻辑
  const handleLogout = async () => {
    try {
      await authApi.signOut(); // 调用后端接口，清除 session
    } catch (e) {
      // 可以忽略错误，保证本地也能退出
    }
    setToken("");
    dispatch(viewerModel.actions.setCredentials({ credentials: null as Nullable<any> }));
    socket.disconnect();
    navigate("/sign-in");
    handleMenuClose();
  };

  return (
    <aside className="fixed right-0 top-0 bottom-0 w-44 h-screen bg-white/90 dark:bg-[rgba(30,35,42,0.95)] border-l border-gray-200 dark:border-white/10 shadow-2xl flex flex-col items-center py-8 z-40">
      {/* 头像区 */}
      <div className="w-4/5 flex flex-col items-center mb-8">
        <button
          onClick={handleProfileClick}
          className="w-20 h-20 rounded-full overflow-hidden border-4 border-[#8BC34A] hover:border-[#AED581] shadow-lg focus:outline-none focus:ring-2 focus:ring-[#8BC34A] transition-all duration-200"
        >
          <Avatar src={credentials.avatar} size="100%" />
        </button>
        {/* 菜单 */}
        <Menu
          anchorEl={anchorEl}
          open={open}
          onClose={handleMenuClose}
          anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
          transformOrigin={{ vertical: 'top', horizontal: 'center' }}
          MenuListProps={{
            sx: { p: 0 },
          }}
          PaperProps={{
            sx: {
              background: (theme) => theme.palette.mode === 'light' ? '#fff' : 'rgba(30,35,42,0.98)',
              color: (theme) => theme.palette.mode === 'light' ? '#222' : '#fff',
              borderRadius: 2,
              minWidth: 160,
              boxShadow: '0 8px 32px rgba(0,0,0,0.25)',
              fontFamily: 'Bungee, sans-serif',
            },
          }}
        >
          <MenuItem
            onClick={handleLogout}
            sx={{
              borderRadius: 2,
              mx: 0.5,
              my: 0.5,
              color: 'inherit',
              fontWeight: 700,
              fontFamily: 'Bungee, sans-serif',
              letterSpacing: 1.5,
              transition: 'background 0.2s',
              '&:hover': {
                bgcolor: '#8BC34A',
                color: '#222',
              },
            }}
          >
            <LogOut style={{ marginRight: 8 }} /> Sign out
          </MenuItem>
        </Menu>
      </div>
      {/* 分割线 */}
      <div className="w-full h-0.5 bg-gray-200 dark:bg-white/20 mb-6" />
      {/* 好友区 */}
      <div className="flex-1 w-3/4 flex flex-col items-center gap-4 overflow-y-auto">
        {!hasFriends && <NoFriendsIcon />}
        {hasFriends &&
          [...friendsWithInterim!]
            .sort((a, b) => {
              const isOnlineA = a.interim?.status === "online";
              const isOnlineB = b.interim?.status === "online";
              return isOnlineA === isOnlineB ? 0 : isOnlineA ? -1 : 1;
            })
            .map((friend) => {
              const status = friend.interim?.status;
              return (
                <Link
                  key={friend.id}
                  to={`/@/${friend.username}`}
                  className="block w-full aspect-square rounded-full overflow-hidden border-2 border-white/20 hover:border-[#8BC34A] transition-all duration-200 shadow-md group"
                >
                  <Avatar
                    src={friend.avatar}
                    size="100%"
                    status={status}
                    showStatus={!!status}
                  />
                </Link>
              );
            })}
      </div>
    </aside>
  );
};

// 无好友时的 Paw 图标和提示
const NoFriendsIcon = () => (
  <div className="flex flex-col items-center justify-center py-8 w-full">
    <PawPrint className="w-16 h-16 text-[#8BC34A] opacity-70 animate-bounce mb-2" />
    <div className="mt-2 text-base font-miriam text-gray-500 dark:text-white/70 tracking-wider text-center select-none">
      You have no friends
    </div>
  </div>
);
