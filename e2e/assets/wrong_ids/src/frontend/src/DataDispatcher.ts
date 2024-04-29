import { ItemDB } from "./data/Data";

// export const AppData = process.env.REACT_APP_USE_MOCK_DATA === "1" ? MockData : ItemDB;
export const AppData = ItemDB; // TODO: Remove this indirection.