import * as React from "react";
import { useEffect, useState } from "react";
import { Button } from "react-bootstrap";
import { useNavigate, useParams } from "react-router-dom";
import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';
import { Helmet } from 'react-helmet';
import 'react-tabs/style/react-tabs.css';
import { idlFactory as mainIdlFactory } from "../../../declarations/main";
import { ItemDataWithoutOwner, ZonBackend } from "../../../declarations/main/main.did";
import { idlFactory as canDBPartitionIdlFactory } from "../../../declarations/CanDBPartition";
import { CanDBPartition } from "../../../declarations/CanDBPartition/CanDBPartition.did";
import EditFoldersList from "./EditFoldersList";
import { parseItemRef, serializeItemRef } from "../data/Data";
import { addToMultipleFolders } from "../util/folder";
import { AuthContext } from "./auth/use-auth-client";
import { BusyContext } from "./App";
import { Actor } from "@dfinity/agent";

export default function EditItem(props: {itemId?: string, comment?: boolean}) {
    const routeParams = useParams();
    const navigate = useNavigate();
    const [mainFolder, setMainFolder] = useState<string | undefined>(undefined); // TODO: For a comment, it may be not a folder.
    const [foldersList, setFoldersList] = useState<[string, 'beginning' | 'end'][]>([]);
    const [antiCommentsList, setAntiCommentsList] = useState<[string, 'beginning' | 'end'][]>([]);
    useEffect(() => {
        setMainFolder(routeParams.folder);
    }, [routeParams.folder]);
    enum FolderKind { owned, communal };
    const [folderKind, setFolderKind] = useState<FolderKind>(FolderKind.owned);
    const [locale, setLocale] = useState('en'); // TODO: user's locale
    const [title, setTitle] = useState("");
    const [shortDescription, setShortDescription] = useState("");
    const [link, setLink] = useState(""); // TODO: Check URL validity.
    const [post, setPost] = useState("");
    enum SelectedTab {selectedLink, selectedOther}
    const [selectedTab, setSelectedTab] = useState(SelectedTab.selectedLink);
    function onSelectTab(index) {
        switch (index) {
            case 0:
                setSelectedTab(SelectedTab.selectedLink);
                break;
            case 1:
                setSelectedTab(SelectedTab.selectedOther);
                break;
            }
    }
    return (
            <BusyContext.Consumer>
                {({setBusy}) =>
                <AuthContext.Consumer>
                    {({agent, defaultAgent, isAuthenticated}) => {
                    async function submit() {
                        useEffect(() => {
                            if (props.itemId !== undefined) {
                                const itemId = parseItemRef(props.itemId);
                                const actor: CanDBPartition = Actor.createActor(canDBPartitionIdlFactory, {canisterId: itemId.canister, agent: defaultAgent});
                                actor.getItem(BigInt(itemId.id))
                                    .then((item0) => {
                                        const item1 = item0[0]!; // FIXME: `!`
                                        const item = item1.data.item;
                                        setFolderKind(item1.communal ? FolderKind.communal : FolderKind.owned);
                                        setLocale(item.locale);
                                        setTitle(item.title);
                                        setShortDescription(item.description);
                                        setLink((item.details as any).link);
                                    });
                                // TODO: Don't call it on non-blogpost:
                                actor.getAttribute({sk: "i/" + itemId.id}, "t")
                                    .then(item1 => {
                                        const text = item1[0]! as any;
                                        if (text !== undefined) {
                                            setPost(text.text);
                                            setSelectedTab(SelectedTab.selectedOther);
                                        }
                                    });
                            }
                        }, [props.itemId]);
                        function itemData(): ItemDataWithoutOwner {
                            // TODO: Differentiating post and message by `post === ""` is unreliable.
                            const isPost = selectedTab == SelectedTab.selectedOther && post !== "";
                            return {
                                // communal: false, // TODO: Item can be communal.
                                locale,
                                title,
                                description: shortDescription,
                                details: selectedTab == SelectedTab.selectedLink ? {link: link} :
                                    isPost ? {post: null} : {message: null},
                                price: 0.0, // TODO
                            };
                        }
                        async function submitItem(item: ItemDataWithoutOwner) {
                            const backend: ZonBackend = Actor.createActor(mainIdlFactory, {canisterId: process.env.CANISTER_ID_MAIN!, agent});
                            let part, n;
                            if (routeParams.item !== undefined) {
                                console.log("routeParams.item", routeParams.item    )
                                const folder = parseItemRef(routeParams.item); // TODO: not here
                                await backend.setItemData(folder.canister, BigInt(folder.id), item);
                                part = folder.canister;
                                n = BigInt(folder.id);
                            } else {
                                [part, n] = await backend.createItemData({data: item, communal: folderKind == FolderKind.communal});
                            }
                            await backend.setPostText(part, n, post);
                            const ref = serializeItemRef({canister: part, id: Number(n)});
                            // TODO: What to do with this on editing the folder?
                            await addToMultipleFolders(agent!, foldersList, {canister: part, id: Number(n)}, false);
                            await addToMultipleFolders(agent!, antiCommentsList, {canister: part, id: Number(n)}, true);
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
                        const folder = parseItemRef(props.itemId!); // TODO: not here
                        await backend.removeItem(folder.canister, BigInt(folder.id));
                        navigate("/");
                    }
                    return <>
                        <Helmet>
                            <title>Zon Social Media - create a new item</title>
                        </Helmet>
                        <p>Language: <input type="text" required={true} defaultValue={locale} onChange={e => setLocale(e.target.value)}/></p>
                        <p>Title: <input type="text" required={true} defaultValue={title} onChange={e => setTitle(e.target.value)}/></p>
                        <p>Short (meta) description: <textarea defaultValue={shortDescription
                        } onChange={e => setShortDescription(e.target.value)}/></p>
                        {/* TODO (should not because complicates ordering?):
                        <p>Link type:
                            <label><input type="radio" name="kind" value="0" required={true}/> Directory entry</label>
                            <label><input type="radio" name="kind" value="1" required={true}/> Message</label></p>*/}
                        <Tabs onSelect={onSelectTab} selectedIndex={selectedTab === SelectedTab.selectedLink ? 0 : 1}>
                            <TabList>
                                <Tab>Link</Tab>
                                <Tab>Blog post</Tab>
                            </TabList>
                            <TabPanel>
                                <p>Link: <input type="url" defaultValue={link} onChange={e => setLink(e.target.value)}/></p>
                            </TabPanel>
                            <TabPanel>
                                <p>Text: <textarea style={{height: "10ex"}} defaultValue={post} onChange={e => setPost(e.target.value)}/></p>
                            </TabPanel>
                        </Tabs>
                        <EditFoldersList
                            defaultFolders={!(props.comment === true) && mainFolder !== undefined ? [[mainFolder, 'beginning']] : []}
                            defaultAntiComments={props.comment === true && mainFolder !== undefined ? [[mainFolder, 'beginning']] : []}
                            onChangeFolders={setFoldersList}
                            onChangeAntiComments={setAntiCommentsList}
                        />
                        <p>
                            <Button onClick={submit} disabled={!isAuthenticated}>Submit</Button>
                            {props.itemId !== undefined &&
                                <Button onClick={remove} disabled={!isAuthenticated}>Delete</Button>
                            }
                        </p>
                    </>;
                }}
            </AuthContext.Consumer>
            }
        </BusyContext.Consumer>
    );
}