import { useEffect, useState } from "react";
import { useCurrentAccount, useSuiClient } from "@mysten/dapp-kit";
import { Transaction } from "@mysten/sui/transactions";
import { useBetterSignAndExecuteTransaction } from "./useBetterTx";
import { toast } from "react-toastify";

export const usePassport = () => {
  const [hasPassport, setHasPassport] = useState<boolean | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

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
    if (!account?.address) return;

    try {
      const objects = await client.getOwnedObjects({
        owner: account.address,
        filter: {
          MatchAll: [
            {
              Package: process.env.NEXT_PUBLIC_TESTNET_PACKAGE!,
            },
            {
              MoveModule: {
                module: "passport",
                package: process.env.NEXT_PUBLIC_TESTNET_PACKAGE!,
              },
            },
          ],
        },
        options: {
          showContent: true,
        },
      });

      setHasPassport(objects.data.length > 0);
    } catch (err) {
      console.error("Failed to check passport:", err);
      setError("Failed to check passport status");
      toast.error("Failed to check passport status");
    }
  };

  // Create new user
  const createNewUser = async () => {
    if (!account?.address) return false;

    setIsCreating(true);
    setError(null);
    setSuccess(false);

    try {
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
          toast.success("Passport created successfully!");
          await checkPassport();
          setSuccess(true);
        })
        .onError((err) => {
          console.error("Failed to create passport:", err);
          setError(err.message || "Failed to create passport");
          toast.error("Failed to create passport");
          setSuccess(false);
        })
        .execute();

      return success;
    } catch (err: any) {
      console.error("Failed to create new user:", err);
      setError(err.message || "Failed to create user");
      toast.error(err.message || "Failed to create user");
      return false;
    } finally {
      setIsCreating(false);
    }
  };

  useEffect(() => {
    if (account?.address) {
      checkPassport();
    }
  }, [account?.address]);

  return {
    hasPassport,
    isCreating: isLoading || isCreating,
    error,
    createNewUser,
    checkPassport,
  };
};
