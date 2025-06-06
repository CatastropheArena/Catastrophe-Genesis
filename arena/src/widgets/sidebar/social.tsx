import React, { useCallback } from "react";
import { useSelector } from "react-redux";
import { Link, useNavigate } from "react-router-dom";
import { Cat, PawPrint, UserX, LogOut, Copy } from "lucide-react";
import { Button, Menu, MenuItem, IconButton } from "@mui/material";
import { useDispatch } from "@app/store";
import { useAuthStore } from 'src/components/auth';
import { socket } from "@shared/lib/ws";
import { Nullable } from "@shared/lib/typings";
import { viewerModel } from "@entities/viewer";
import { userModel } from "@entities/user";
import { Avatar } from "@shared/ui/atoms";
import { User, UserWithInterim } from "@shared/api/common";
import { authApi } from "@shared/api/auth";
import { authModel } from "@features/auth";
import { useSnackbar } from "notistack";

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
  const { enqueueSnackbar } = useSnackbar();

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

  // 添加复制ID的处理函数
  const handleCopyId = useCallback(() => {
    if (credentials?.id) {
      navigator.clipboard.writeText(credentials.id);
      enqueueSnackbar("用户ID已复制到剪贴板", { variant: "success" });
      handleMenuClose();
    }
  }, [credentials?.id, enqueueSnackbar]);

  // 如果未登录，不渲染任何内容
  if (!credentials) {
    return null;
  }

  // 使用 useCallback 优化退出登录函数
  const handleLogout = useCallback(async () => {
    try {
      // 1. 先关闭菜单，避免UI状态影响后续操作
      handleMenuClose();

      // 2. 先断开 socket 连接
      socket.disconnect();

      // 3. 调用后端登出接口并更新认证状态
      const response = await dispatch(authModel.actions.signOut()).unwrap();
      
      if (!response.success) {
        throw new Error(response.message || "退出登录失败");
      }

      // 4. 清除用户数据（创建一个空的用户对象，保持类型一致性）
      const emptyUser: User = {
        id: "",
        username: "",
        avatar: "",
        rating: 0
      };
      dispatch(viewerModel.actions.setCredentials({ credentials: emptyUser }));

      // 5. 最后清除 token
      setToken("");

      // 6. 跳转到登录页面（使用 replace 避免返回）
      navigate("/sign-in", { replace: true });
    } catch (error) {
      console.error("Logout error:", error);
      // 显示错误提示
      enqueueSnackbar(error instanceof Error ? error.message : "退出登录失败", {
        variant: "error",
      });
      // 即使报错也要确保用户能退出
      socket.disconnect();
      dispatch(viewerModel.actions.setCredentials({ 
        credentials: {
          id: "",
          username: "",
          avatar: "",
          rating: 0
        } 
      }));
      setToken("");
      navigate("/sign-in", { replace: true });
    }
  }, [dispatch, setToken, navigate, enqueueSnackbar]);

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
            onClick={handleCopyId}
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
            <Copy style={{ marginRight: 8 }} /> Copy ID
          </MenuItem>
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
