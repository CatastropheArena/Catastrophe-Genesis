import React from "react";
import {useSelector} from "react-redux";

import {useDispatch} from "@app/store";
import {viewerModel} from "@entities/viewer";

import {model} from "../model";
import { useAuthStore } from "src/components/auth";

export interface CredentialsObtainerProps {
  children: React.ReactNode;
}

export const CredentialsObtainer: React.FC<CredentialsObtainerProps> = ({
  children,
}) => {
  const dispatch = useDispatch();

  const credentials = viewerModel.useCredentials();
  const token = useAuthStore((state) => state.token);
  const isFetching = useSelector(model.selectors.areCredentialsFetching);

  React.useEffect(() => {
    if (token && !credentials) {
      dispatch(model.actions.setAreCredentialsFetching({areFetching: true}));
      dispatch(model.actions.fetchCredentials())
        .unwrap()
        .then((res) => {
          dispatch(
            viewerModel.actions.setCredentials({credentials: res.credentials}),
          );
        })
        .finally(() => {
          dispatch(
            model.actions.setAreCredentialsFetching({areFetching: false}),
          );
        });
    }
  }, [token, credentials]);

  if (isFetching) return null;

  return <>{children}</>;
};
