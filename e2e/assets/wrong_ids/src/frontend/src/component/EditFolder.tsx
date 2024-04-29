import * as React from "react";
import { useEffect, useState } from "react";
import { Button } from "react-bootstrap";
import { useNavigate, useParams } from "react-router-dom";
import { Tab, TabList, TabPanel, Tabs } from "react-tabs";
import { Helmet } from 'react-helmet';
import { idlFactory as mainIdlFactory } from "../../../declarations/main";
import { ItemDataWithoutOwner, ItemTransferWithoutOwner, ZonBackend } from "../../../declarations/main/main.did";
import { idlFactory as canDBPartitionIdlFactory } from "../../../declarations/CanDBPartition";
import { CanDBPartition } from "../../../declarations/CanDBPartition/CanDBPartition.did";
import EditFoldersList from "./EditFoldersList";
import { addToFolder, addToMultipleFolders } from "../util/folder";
import { parseItemRef, serializeItemRef } from "../data/Data";
import { AuthContext } from "./auth/use-auth-client";
import { BusyContext } from "./App";
import { Actor, Agent } from "@dfinity/agent";

export default function EditFolder(props: {super?: boolean, folderId?: string, superFolderId?: string, defaultAgent: Agent | undefined}) {
    const navigate = useNavigate();
    const [superFolder, setSuperFolder] = useState<string | undefined>();
    const [foldersList, setFoldersList] = useState<[string, 'beginning' | 'end'][]>([]);
    const [antiCommentsList, setAntiCommentsList] = useState<[string, 'beginning' | 'end'][]>([]);
    useEffect(() => {
        setSuperFolder(props.superFolderId);
    }, [props.superFolderId]);
    enum FolderKind { owned, communal };
    const [folderKind, setFolderKind] = useState<FolderKind>(FolderKind.owned);
    const [locale, setLocale] = useState('en'); // TODO: user's locale
    const [title, setTitle] = useState("");
    const [shortDescription, setShortDescription] = useState("");
    useEffect(() => {
        if (props.folderId !== undefined) {
            const folderId = parseItemRef(props.folderId);
            const actor: CanDBPartition = Actor.createActor(canDBPartitionIdlFactory, {canisterId: folderId.canister, agent: props.defaultAgent});
            actor.getItem(BigInt(folderId.id))
                .then((itemx) => {
                    const item = itemx[0] ? itemx[0][0]!.data : undefined;
                    const communal = itemx[0]?.communal; // TODO: Simplify.
                    setFolderKind(communal ? FolderKind.communal : FolderKind.owned);
                    setLocale(item!.locale);
                    setTitle(item!.title);
                    setShortDescription(item!.description);
                });
        }
    }, [props.folderId]);
    function onSelectTab(index: number) {
        switch (index) {
            case 0:
                setFolderKind(FolderKind.owned);
                break;
            case 1:
                setFolderKind(FolderKind.communal);
                break;
            }
    }
    return (
        <BusyContext.Consumer>
        {({setBusy}) =>
            <AuthContext.Consumer>
            {({agent, isAuthenticated}) => {
                async function submit() {
                    function itemData(): ItemDataWithoutOwner {
                        return {
                            locale,
                            title,
                            description: shortDescription,
                            details: {folder: null},
                            price: 0.0, // TODO
                        };
                    }
                    async function submitItem(item: ItemDataWithoutOwner) {
                        const backend: ZonBackend = Actor.createActor(mainIdlFactory, {canisterId: process.env.CANISTER_ID_MAIN!, agent});
                        let part, n;
                        if (props.folderId !== undefined) {
                            const folder = parseItemRef(props.folderId); // TODO: not here
                            await backend.setItemData(folder.canister, BigInt(folder.id), item);
                            part = folder.canister;
                            n = BigInt(folder.id);
                        } else {
                            const transfer: ItemTransferWithoutOwner = {data: item, communal: folderKind == FolderKind.communal};
                            [part, n] = await backend.createItemData(transfer);
                        }
                        const ref = serializeItemRef({canister: part, id: Number(n)}); // TODO: Reduce code
                        if (!(props.super === true)) { // noComments
                            await addToMultipleFolders(agent!, foldersList, {canister: part, id: Number(n)}, false);
                            await addToMultipleFolders(agent!, antiCommentsList, {canister: part, id: Number(n)}, true);
                        } else {
                            for (const folder of foldersList) {
                                // TODO: It may fail to parse.
                                await addToFolder(agent!, {canister: part, id: Number(n)}, parseItemRef(folder[0]), false, folder[1]);
                            }
                        }
                        navigate("/item/"+ref);
                    }
                    setBusy(true);
                    await submitItem(itemData());
                    setBusy(false);
                }
                async function remove() {
                    if (!window.confirm("Really delete?")) {
                        return;
                    }
                    const backend: ZonBackend = Actor.createActor(mainIdlFactory, {canisterId: process.env.CANISTER_ID_MAIN!, agent});
                    const folder = parseItemRef(props.folderId!); // TODO: not here
                    await backend.removeItem(folder.canister, BigInt(folder.id));
                    navigate("/");
                }
                return <>
                    <Helmet>
                        <title>Zon Social Media - create a new folder</title>
                    </Helmet>
                    <h1>{props.folderId !== undefined ? `Edit folder` :
                        props.super === true ? `Create superfolder` : `Create subfolder`}</h1>
                    <Tabs onSelect={onSelectTab}>
                        <TabList>
                            <Tab>Owned</Tab>
                            <Tab>Communal</Tab>
                        </TabList>
                        <TabPanel>
                            <p>Owned folders have an owner (you). Only the owner can add, delete, and reoder items in an owned folder,{" "}
                                or rename the folder.</p>
                        </TabPanel>
                        <TabPanel>
                            <p>Communal folders have no owner. Anybody can add an item to a communal folder.{" "}
                                Nobody can delete an item from a communal folder or rename the folder. Ordering is determined by voting.</p>
                        </TabPanel>
                    </Tabs>
                    <p>Language: <input type="text" required={true} defaultValue={locale} onChange={e => setLocale(e.target.value)}/></p>
                    <p>Title: <input type="text" required={true} defaultValue={title} onChange={e => setTitle(e.target.value)}/></p>
                    <p>Short (meta) description: <textarea defaultValue={shortDescription} onChange={e => setShortDescription(e.target.value)}/></p>
                    <EditFoldersList
                        defaultFolders={superFolder === undefined ? [] : [[superFolder, 'beginning']]}
                        onChangeFolders={setFoldersList}
                        onChangeAntiComments={setAntiCommentsList}
                        reverse={props.super === true}
                        noComments={props.super === true}
                    />
                    <p>
                        <Button onClick={submit} disabled={!isAuthenticated}>Save</Button>{" "}
                        {props.folderId !== undefined &&
                            <Button onClick={remove} disabled={!isAuthenticated}>Delete</Button>
                        }
                    </p>
                </>
            }}
            </AuthContext.Consumer>
        }
        </BusyContext.Consumer>
    );
}