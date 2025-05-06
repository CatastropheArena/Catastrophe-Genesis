import {createRoot} from "react-dom/client";
import {BrowserRouter} from "react-router-dom";
import {Provider} from "react-redux";

import {App} from "@app/client";
import {store} from "@app/store";
import "@shared/lib/i18n";
import { StrictMode } from "react";
import {Suiet, SuiWeb3ConfigProvider} from '@ant-design/web3-sui';
import {networkConfig} from "./config/networkConfig";
import {NETWORK} from "./config/constants";
import { createSyncStoragePersister } from '@tanstack/query-sync-storage-persister';
import { QueryClient } from '@tanstack/react-query';
import { PersistQueryClientProvider } from '@tanstack/react-query-persist-client';

const element = document.getElementById("root") as Element;
const root = createRoot(element);
const queryClient = new QueryClient();
const persister = createSyncStoragePersister({
  storage: typeof window !== 'undefined' ? window.localStorage : undefined,
});
root.render(
  <StrictMode>
    <Provider store={store}>
      <PersistQueryClientProvider client={queryClient} persistOptions={{ persister }}>
        <SuiWeb3ConfigProvider
            wallets={[Suiet()]}
            networkConfig={networkConfig}
            sns={true}
            autoConnect={true}
            defaultNetwork={NETWORK}
        >
          <BrowserRouter>
            <App />
          </BrowserRouter>
        </SuiWeb3ConfigProvider>
      </PersistQueryClientProvider>
    </Provider>
  </StrictMode>,
);
