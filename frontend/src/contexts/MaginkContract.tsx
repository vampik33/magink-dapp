import { PropsWithChildren, createContext } from "react";
import { useContract, DryRun, useDryRun, useTx, Tx, useCall, Call, ChainContract } from "useink";
import { CONTRACT_ADDRESS } from "../const";
import metadata from "../metadata.json";
import { useTxNotifications } from "useink/notifications";

interface MaginkContractState {
  magink?: ChainContract; 
  startDryRun?: DryRun<number>;
  claimDryRun?: DryRun<number>;
  start?: Tx<number>;
  claim?: Tx<number>;
  getRemaining?: Call<number>;
  getRemainingFor?: Call<number>;
  getBadges?: Call<number>;
  getBadgesFor?: Call<number>;
  mint_wizard?: Tx<number>;
}

export const MaginkContractContext = createContext<MaginkContractState>({});

export function MaginkContractProvider({ children }: PropsWithChildren) {
  const magink = useContract(CONTRACT_ADDRESS, metadata);
  const claimDryRun = useDryRun<number>(magink, 'claim');
  const startDryRun = useDryRun<number>(magink, 'start');
  const MintWizardDryRun = useDryRun<number>(magink, 'mint_wizard');
  const claim = useTx(magink, 'claim');
  const start = useTx(magink, 'start');
  const mint_wizard = useTx(magink, 'mint_wizard');
  const getRemaining = useCall<number>(magink, 'getRemaining');
  const getBadges = useCall<number>(magink, 'getBadges');
  const getBadgesFor = useCall<number>(magink, 'getBadgesFor');
  const getRemainingFor = useCall<number>(magink, 'getRemainingFor');
  useTxNotifications(claim);
  useTxNotifications(start);
  useTxNotifications(mint_wizard);

  return (
    <MaginkContractContext.Provider value={{ magink, startDryRun, claimDryRun, start, claim, getRemaining, getRemainingFor, getBadges, getBadgesFor, mint_wizard }}>
      {children}
    </MaginkContractContext.Provider>
  );
}