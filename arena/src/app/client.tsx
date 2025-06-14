import React from "react";

import {ViewerProfileHandler} from "@entities/viewer";
import {UserEventsHandler} from "@entities/user";
import {CredentialsObtainer} from "@features/auth";
import {MatchRejoinBoundary} from "@features/match-rejoin";
import {
  MatchmakingQueueHandler,
  MatchmakingQueueIndicator,
} from "@features/matchmaking-queue";
import {LobbyHandler, LobbyIndicator} from "@features/lobby-rejoin";
import {Routes} from "@pages/routes";
import {ThemingProvider} from "@shared/lib/theming";
import {NotificationProvider} from "@shared/lib/notification";
import {GlobalStyles} from "./global-styles";
import {DesktopOnlyRestrict} from "./desktop-only";
import { SettingsSidebar } from "./settings-sidebar";
import { ConfigProvider} from 'antd';
const styles = <GlobalStyles />;
export const App: React.FC = () => (
  <ThemingProvider>
  <ConfigProvider
      theme={{
        token: {
          colorPrimary: '#8BC34A',
        },
        components: {
          Dropdown: {
            colorBgElevated: 'rgba(76, 175, 80, 0.8)',
            colorText: 'primary',
            algorithm: true,
          }
        }
      }}
      >
    {styles}
    <React.Suspense>
      <NotificationProvider>
        <DesktopOnlyRestrict>
          <CredentialsObtainer>
            {/* <ViewerProfileHandler>
              <UserEventsHandler>
                <MatchmakingQueueHandler>
                  <MatchmakingQueueIndicator />
                  <MatchRejoinBoundary>
                    <LobbyHandler>
                      <LobbyIndicator /> */}
                      <SettingsSidebar>
                        <Routes />
                      </SettingsSidebar>
                    {/* </LobbyHandler>
                  </MatchRejoinBoundary>
                </MatchmakingQueueHandler>
              </UserEventsHandler>
            </ViewerProfileHandler> */}
          </CredentialsObtainer>
        </DesktopOnlyRestrict>
      </NotificationProvider>
    </React.Suspense>
    </ConfigProvider>
  </ThemingProvider>
);
