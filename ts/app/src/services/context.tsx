// The services of the document for every screen. The provider creates them
// once and installs the navigation gate, which listens before the router.

import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { router } from "expo-router";

import { installNavigationGate, type GateWindow } from "../state/navigation.ts";
import { browserEnvironment, linkOf } from "./browser.ts";
import { earlyHistory } from "./early-history.ts";
import { createAppServices, type AppServices } from "./services.ts";

const ServicesContext = createContext<AppServices | null>(null);

export function ServicesProvider({ children }: { children: ReactNode }) {
  const [services] = useState(() =>
    createAppServices(
      browserEnvironment({
        push: (href) => router.push(href),
        replace: (href) => router.replace(href),
      }),
    ),
  );
  useEffect(() => {
    // History moves reach the gate through the listener that the browser
    // calls before the router's own.
    const history: GateWindow = {
      location: window.location,
      addEventListener: (_type, listener) => earlyHistory.add(listener),
      removeEventListener: (_type, listener) => earlyHistory.remove(listener),
    };
    return installNavigationGate(services.gate, history, document, linkOf);
  }, [services]);
  return (
    <ServicesContext.Provider value={services}>
      {children}
    </ServicesContext.Provider>
  );
}

export function useServices(): AppServices {
  const services = useContext(ServicesContext);
  if (services === null) throw new Error("ServicesProvider is missing.");
  return services;
}
