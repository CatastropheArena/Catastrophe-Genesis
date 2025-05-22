import { useEffect, useState } from "react";
import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";
import { Transaction } from "@mysten/sui/transactions";
import { useBetterSignAndExecuteTransaction } from "./useBetterTx";
import { toast } from "react-toastify";
import { useUserAssets } from "./useUserAssets";
import { SuiObjectData, SuiTransactionBlockResponse } from "@mysten/sui/client";

export const usePassport = () => {
  const [hasPassport, setHasPassport] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [isChecking, setIsChecking] = useState(false);
  const { assets, fetchAssets } = useUserAssets();
  const [passport, setPassport] = useState<Passport | null>(null);

  const client = useSuiClient();
  const account = useCurrentAccount();
  const { handleSignAndExecuteTransaction, isLoading } =
    useBetterSignAndExecuteTransaction({
      tx: () => {
        const tx = new Transaction();
        tx.moveCall({
          target: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::user::create_new_user`,
          arguments: [
            tx.object(`${process.env.NEXT_PUBLIC_TESTNET_PASSPORT_STATE}`),
            tx.object(`${process.env.NEXT_PUBLIC_TESTNET_TREASURY}`),
            tx.object(`${process.env.NEXT_PUBLIC_TESTNET_FRAGMENT_STORE}`),
            tx.object("0x6"),
          ],
        });
        return tx;
      },
      options: {
        showEffects: true,
        showObjectChanges: true,
      },
    });

  // Check if user has passport
  const checkPassport = async () => {
    if (!account?.address) {
      setHasPassport(null);
      setPassport(null);
      return;
    }

    setIsChecking(true);
    try {
      const objects = await client.getOwnedObjects({
        owner: account.address,
        options: {
          showContent: true,
        },
        filter: {
          MatchAny: [
            {
              StructType: `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::passport::Passport`,
            },
          ],
        },
      });

      let foundPassport = false;
      objects.data.forEach((object) => {
        const data = object.data as unknown as SuiObjectData;
        if (data.content?.dataType !== "moveObject") {
          return;
        }
        const contentType = data.content?.type;
        if (
          contentType ===
          `${process.env.NEXT_PUBLIC_TESTNET_PACKAGE}::passport::Passport`
        ) {
          foundPassport = true;
          setHasPassport(true);
          setPassport(data.content?.fields as unknown as Passport);
        }
      });

      if (!foundPassport) {
        setHasPassport(false);
        setPassport(null);
      }
    } catch (err) {
      console.error("Failed to check passport:", err);
      setError("Failed to check passport status");
      toast.error("Failed to check passport status");
      setHasPassport(null);
    } finally {
      setIsChecking(false);
    }
  };

  // Create new user
  const createNewUser = async () => {
    if (!account?.address) return false;

    setError(null);
    setSuccess(false);

    try {
      const result: void | SuiTransactionBlockResponse =
        await handleSignAndExecuteTransaction()
          .beforeExecute(async () => {
            // Check passport status again before executing transaction
            await checkPassport();
            if (hasPassport) {
              toast.info("You already have a passport");
              return false;
            }
            return true;
          })
          .onSuccess(async () => {
            // First update passport status
            await checkPassport();

            // Then update assets and set success state
            await fetchAssets();
            setSuccess(true);
            toast.success("Passport created successfully!");
          })
          .onError((err) => {
            console.error("Failed to create passport:", err);
            setError(err.message || "Failed to create passport");
            toast.error(err.message || "Failed to create passport");
            setSuccess(false);
          })
          .execute();

      return result?.effects?.status?.status === "success";
    } catch (err: any) {
      console.error("Failed to create new user:", err);
      setError(err.message || "Failed to create passport");
      toast.error(err.message || "Failed to create passport");
      return false;
    }
  };

  // Check passport when wallet changes
  useEffect(() => {
    checkPassport();
  }, [account?.address]);

  return {
    hasPassport,
    passport,
    isCreating: isLoading,
    isChecking,
    error,
    success,
    createNewUser,
    checkPassport,
  };
};
