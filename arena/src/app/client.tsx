import React from "react";

// import {ViewerProfileHandler} from "@entities/viewer";
// import {UserEventsHandler} from "@entities/user";
// import {CredentialsObtainer} from "@features/auth";
// import {MatchRejoinBoundary} from "@features/match-rejoin";
// import {
//   MatchmakingQueueHandler,
//   MatchmakingQueueIndicator,
// } from "@features/matchmaking-queue";
// import {LobbyHandler, LobbyIndicator} from "@features/lobby-rejoin";
// import {Routes} from "@pages/routes";
import {ThemingProvider} from "@shared/lib/theming";
import {NotificationProvider} from "@shared/lib/notification";
import {GlobalStyles} from "./global-styles";
import {DesktopOnlyRestrict} from "./desktop-only";
import { SettingsSidebar } from "./settings-sidebar";
import { SignInPage } from "@pages/auth/sign-in";
import { SignUpPage } from "@pages/auth/sign-up";
const styles = <GlobalStyles />;

export const App: React.FC = () => (
  <ThemingProvider>
    {styles}
    <React.Suspense>
      <SettingsSidebar>
      <NotificationProvider>
        <DesktopOnlyRestrict>
          {/* <CredentialsObtainer> */}
            {/* <ViewerProfileHandler> */}
              {/* <UserEventsHandler> */}
                {/* <MatchmakingQueueHandler> */}
                  {/* <MatchmakingQueueIndicator /> */}
                  {/* <MatchRejoinBoundary> */}
                    {/* <LobbyHandler> */}
                      {/* <LobbyIndicator /> */}
                      {/* <Routes /> */}
                      <SignInPage />
                      {/* <SignUpPage/> */}
                    {/* </LobbyHandler> */}
                  {/* </MatchRejoinBoundary> */}
                {/* </MatchmakingQueueHandler> */}
              {/* </UserEventsHandler> */}
            {/* </ViewerProfileHandler> */}
          {/* </CredentialsObtainer> */}
        </DesktopOnlyRestrict>
      </NotificationProvider>
      </SettingsSidebar>
    </React.Suspense>
  </ThemingProvider>
);
