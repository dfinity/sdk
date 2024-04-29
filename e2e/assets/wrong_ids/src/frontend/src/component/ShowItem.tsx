import * as React from "react";
import { useContext, useEffect, useState } from "react";
import { AppData } from "../DataDispatcher";
import { Link, useNavigate, useParams } from "react-router-dom";
import { AuthContext } from "./auth/use-auth-client";
import { ItemDB, ItemRef, loadTotalVotes, loadUserVote, parseItemRef, serializeItemRef } from "../data/Data";
import ItemType from "./misc/ItemType";
import { Button, Nav } from "react-bootstrap";
import { ItemData, ItemTransfer } from "../../../declarations/CanDBPartition/CanDBPartition.did";
import UpDown, { updateVotes } from "./misc/UpDown";
import { Tab, TabList, TabPanel, Tabs } from "react-tabs";
import { Helmet } from 'react-helmet';
import { Agent } from "@dfinity/agent";

export default function ShowItem() {
    return (
        <>
            <AuthContext.Consumer>
                {({defaultAgent}) => {
                    return <ShowItemContent defaultAgent={defaultAgent}/>
                }}
            </AuthContext.Consumer>
        </>
    );
}

function ShowItemContent(props: {defaultAgent: Agent | undefined}) {
    const { id: idParam } = useParams();
    const [id, setId] = useState(parseItemRef(idParam!));
    useEffect(() => {
        setId(parseItemRef(idParam!))
    }, [idParam]);
    const { principal } = useContext(AuthContext) as any;
    const [locale, setLocale] = useState("");
    const [title, setTitle] = useState("");
    const [description, setDescription] = useState("");
    const [postText, setPostText] = useState("");
    const [type, setType] = useState<string | undefined>(undefined);
    const [creator, setCreator] = useState("");
    const [subfolders, setSubfolders] = useState<{order: string, id: ItemRef, item: ItemTransfer}[] | undefined>(undefined);
    const [superfolders, setSuperfolders] = useState<{order: string, id: ItemRef, item: ItemTransfer}[] | undefined>(undefined);
    const [items, setItems] = useState<{order: string, id: ItemRef, item: ItemTransfer}[] | undefined>(undefined);
    const [comments, setComments] = useState<{order: string, id: ItemRef, item: ItemTransfer}[] | undefined>(undefined);
    const [antiComments, setAntiComments] = useState<{order: string, id: ItemRef, item: ItemTransfer}[] | undefined>(undefined);
    const [data, setData] = useState<ItemTransfer | undefined>(undefined); // TODO: hack
    const [xdata, setXData] = useState<ItemDB | undefined>(undefined); // TODO: hack
    const [itemsLast, setItemsLast] = useState("");
    const [itemsReachedEnd, setItemsReachedEnd] = useState(false);
    const [commentsLast, setCommentsLast] = useState("");
    const [commentsReachedEnd, setCommentsReachedEnd] = useState(false);
    const [antiCommentsLast, setAntiCommentsLast] = useState("");
    const [antiCommentsReachedEnd, setAntiCommentsReachedEnd] = useState(false);
    const [streamKind, setStreamKind] = useState<"t" | "v">("v"); // time, votes
    const [totalVotesSubFolders, setTotalVotesSubFolders] = useState<{[key: string]: {up: number, down: number}}>({});
    const [userVoteSubFolders, setUserVoteSubFolders] = useState<{[key: string]: number}>({});
    const [totalVotesSuperFolders, setTotalVotesSuperFolders] = useState<{[key: string]: {up: number, down: number}}>({});
    const [userVoteSuperFolders, setUserVoteSuperFolders] = useState<{[key: string]: number}>({});
    const [totalVotesItems, setTotalVotesItems] = useState<{[key: string]: {up: number, down: number}}>({});
    const [userVoteItems, setUserVoteItems] = useState<{[key: string]: number}>({});
    const [totalVotesComments, setTotalVotesComments] = useState<{[key: string]: {up: number, down: number}}>({});
    const [userVoteComments, setUserVoteComments] = useState<{[key: string]: number}>({});
    const [totalVotesCommentsOn, setTotalVotesCommentsOn] = useState<{[key: string]: {up: number, down: number}}>({});
    const [userVoteCommentsOn, setUserVoteCommentsOn] = useState<{[key: string]: number}>({});

    const navigate = useNavigate();
    useEffect(() => {
        setSubfolders(undefined);
        setSuperfolders(undefined);
        setItems(undefined);
        setComments(undefined);
        setAntiComments(undefined);
    }, [id]);
    // TODO: Are `!`s suitable here?
    useEffect(() => {
        updateVotes(props.defaultAgent!, id, principal, subfolders!, setTotalVotesSubFolders, setUserVoteSubFolders).then(() => {}); // TODO: `!`
    }, [subfolders, principal]);
    useEffect(() => {
        updateVotes(props.defaultAgent!, id, principal, superfolders!, setTotalVotesSuperFolders, setUserVoteSuperFolders).then(() => {}); // TODO: `!`
    }, [superfolders, principal]);
    useEffect(() => {
        updateVotes(props.defaultAgent!, id, principal, items!, setTotalVotesItems, setUserVoteItems).then(() => {}); // TODO: `!`
    }, [items, principal]);
    useEffect(() => {
        updateVotes(props.defaultAgent!, id, principal, comments!, setTotalVotesComments, setUserVoteComments).then(() => {}); // TODO: `!`
    }, [comments, principal]);
    useEffect(() => { // TODO
        if (id !== undefined) {
            console.log("Loading from AppData");
            AppData.create(props.defaultAgent!, serializeItemRef(id), streamKind).then(data => { // TODO: `!`
                setXData(data);
                setData(data.item);
                data.locale().then(x => setLocale(x));
                data.title().then(x => setTitle(x));
                data.description().then(x => setDescription(x));
                data.postText().then(x => setPostText(x!)); // TODO: `!`
                data.creator().then(x => setCreator(x.toString())); // TODO
                data.subFolders().then(x => setSubfolders(x));
                data.superFolders().then(x => {
                    setSuperfolders(x);
                });
                data.items().then(x => {
                    setItems(x);
                    if (x.length !== 0) {
                        setItemsLast(x[x.length - 1].order); // duplicate code
                    }
                });
                data.comments().then(x => {
                    setComments(x);
                    if (x.length !== 0) {
                        setCommentsLast(x[x.length - 1].order); // duplicate code
                    }
                });
                data.antiComments().then(x => {
                    setAntiComments(x);
                    if (x.length !== 0) {
                        setAntiCommentsLast(x[x.length - 1].order); // duplicate code
                    }
                });
                data.details().then((x) => {
                    setType(Object.keys(x)[0]);
                });
            });
        }
    }, [id, props.defaultAgent, streamKind]); // TODO: more tight choice
    function moreSubfolders(event: any) {
        event.preventDefault();
        navigate(`/subfolders-of/`+serializeItemRef(id))
    }
    function moreSuperfolders(event: any) {
        event.preventDefault();
        navigate(`/superfolders-of/`+serializeItemRef(id))
    }
    function moreItems(event: any) {
        event.preventDefault();
        if (items?.length === 0) {
            return;
        }
        const lowerBound = itemsLast + 'x';
        xdata.items({lowerBound, limit: 10}).then(x => {
            setItems(items?.concat(x));
            if (x.length !== 0) {
                setItemsLast(x[x.length - 1].order); // duplicate code
            } else {
                setItemsReachedEnd(true);
            }
        });
    }
    function moreComments(event: any) {
        event.preventDefault();
        if (comments?.length === 0) {
            return;
        }
        const lowerBound = commentsLast + 'x';
        xdata!.items({lowerBound, limit: 10}).then(x => { // TODO: `!`
            setItems(comments?.concat(x));
            if (x.length !== 0) {
                setCommentsLast(x[x.length - 1].order); // duplicate code
            } else {
                setCommentsReachedEnd(true);
            }
        });
    }
    function moreAntiComments(event: any) {
        event.preventDefault();
        if (antiComments?.length === 0) {
            return;
        }
        const lowerBound = antiCommentsLast + 'x';
        xdata!.items({lowerBound, limit: 10}).then(x => { // TODO: `!`
            setItems(antiComments?.concat(x));
            if (x.length !== 0) {
                setAntiCommentsLast(x[x.length - 1].order); // duplicate code
            } else {
                setAntiCommentsReachedEnd(true);
            }
        });
    }
    function updateStreamKind(e) {
        setStreamKind(e.currentTarget.value);
    }
    const isFolder = type === 'folder';
    return <>
        <Helmet>
            <title>{isFolder ? `${title} (folder) - Zon` : `${title} - Zon`}</title>
            <meta name="description" content={description}/>
        </Helmet>
        {/* FIXME: `!` on the next line */}
        <h2><ItemType item={data!}/>{isFolder ? "Folder: " : " "}<span lang={locale}>{title}</span></h2>
        <p>Creator: <small>{creator.toString()}</small></p>
        {description !== null ? <p lang={locale}>{description}</p> : ""}
        {postText !== "" ? <p lang={locale}>{postText}</p> : ""}
        <p>Sort by:{" "}
            <label><input type="radio" name="stream" value="t" onChange={updateStreamKind} checked={streamKind == "t"}/> time</label>{" "}
            <label><input type="radio" name="stream" value="v" onChange={updateStreamKind} checked={streamKind == "v"}/> votes</label>{" "}
        </p>
        <Tabs>
            <TabList>
                <Tab>Main content</Tab>
                <Tab>Comments</Tab>
            </TabList>
            <TabPanel>
                {!isFolder ? "" : <>
                <h3>Sub-folders</h3>
                {subfolders === undefined ? <p>Loading...</p> :
                <ul>
                    {subfolders.map((x: {order: string, id: ItemRef, item: ItemTransfer}) =>
                        <li lang={x.item.data.item.locale} key={serializeItemRef(x.id as any)}>
                             {/* FIXME: `!` below */}
                            <UpDown
                                parent={{id}}
                                item={x}
                                agent={props.defaultAgent!}
                                userVote={userVoteSubFolders[serializeItemRef(x.id)]}
                                totalVotes={totalVotesSubFolders[serializeItemRef(x.id)]}
                                onSetUserVote={(id: ItemRef, v: number) =>
                                    setUserVoteSubFolders({...userVoteSubFolders, [serializeItemRef(id)]: v})}
                                onSetTotalVotes={(id: ItemRef, v: {up: number, down: number}) =>
                                    setTotalVotesSubFolders({...totalVotesSubFolders, [serializeItemRef(id)]: v})}
                                onUpdateList={() => xdata!.subFolders().then(x => setSubfolders(x))}
                            />
                            <ItemType item={x.item}/>
                            <a href={`#/item/${serializeItemRef(x.id)}`}>{x.item.data.item.title}</a>
                            [<Nav.Link href={`#/folder/edit/${serializeItemRef(x.id)}`} style={{display: 'inline'}}>Edit</Nav.Link>]
                        </li>)}
                </ul>}
                <p>
                    <a href="#" onClick={e => moreSubfolders(e)}>More...</a> <a href={`#/create-subfolder/for-folder/${serializeItemRef(id)}`}>Create subfolder</a>
                </p>
            </>}
            <h3>Super-folders</h3>
            <p><small>Voting in this stream not yet implemented.</small></p>
            {superfolders === undefined ? <p>Loading...</p> :
            <ul>
                {superfolders.map((x: {order: string, id: ItemRef, item: ItemTransfer}) =>
                    <li lang={x.item.data.item.locale} key={serializeItemRef(x.id as any)}>
                        {/* TODO: up/down here is complicated by exchanhing parent/child. */}
                        {/*<UpDown
                            parent={{id}}
                            item={x}
                            agent={props.defaultAgent}
                            userVote={userVoteSuperFolders[serializeItemRef(x.id)]}
                            totalVotes={totalVotesSuperFolders[serializeItemRef(x.id)]}
                            onSetUserVote={(id: ItemRef, v: number) =>
                                setUserVoteSuperFolders({...userVoteSuperFolders, [serializeItemRef(id)]: v})}
                            onSetTotalVotes={(id: ItemRef, v: {up: number, down: number}) =>
                                setTotalVotesSuperFolders({...totalVotesSubFolders, [serializeItemRef(id)]: v})}
                            onUpdateList={() => xdata.superFolders().then(x => {
                                console.log(x)
                                setSuperfolders(x);
                            })}
                        />*/}
                        <ItemType item={x.item}/>
                        <a href={`#/item/${serializeItemRef(x.id)}`}>{x.item.data.item.title}</a>
                        [<Nav.Link href={`#/folder/edit/${serializeItemRef(x.id)}`} style={{display: 'inline'}}>Edit</Nav.Link>]
                    </li>)}
            </ul>}
            {/* TODO: Create super-folder */}
            <p><a href="#" onClick={e => moreSuperfolders(e)}>More...</a> <a href={`#/create-superfolder/for-folder/${serializeItemRef(id)}`}>Create</a></p>
            {!isFolder ? "" : <>
                <h3>Items</h3>
                {items === undefined ? <p>Loading...</p> : items.map((x: {order: string, id: ItemRef, item: ItemTransfer}) => 
                    <div key={serializeItemRef(x.id)}>
                        {/* FIXME: `!` in `props.defaultAgent!` */}
                        <p lang={x.item.data.item.locale}>
                            <UpDown
                                parent={{id}}
                                item={x}
                                agent={props.defaultAgent!}
                                userVote={userVoteItems[serializeItemRef(x.id)]}
                                totalVotes={totalVotesItems[serializeItemRef(x.id)]}
                                onSetUserVote={(id: ItemRef, v: number) =>
                                    setUserVoteItems({...userVoteItems, [serializeItemRef(id)]: v})}
                                onSetTotalVotes={(id: ItemRef, v: {up: number, down: number}) =>
                                    setTotalVotesItems({...totalVotesItems, [serializeItemRef(id)]: v})}
                                onUpdateList={() => xdata.items().then(x => setItems(x))}
                            />{" "}
                            {x.item.data.item.price ? <>({x.item.data.item.price} ICP) </> : ""}
                            {(x.item.data.item.details as any).link ? <a href={(x.item.data.item.details as any).link}>{x.item.data.item.title}</a> : x.item.data.item.title}
                            {" "}<a href={`#/item/${serializeItemRef(x.id)}`} title="Homepage">[H]</a>
                            {" "}[<Nav.Link href={`#/item/edit/${serializeItemRef(x.id)}`} style={{display: 'inline'}}>Edit</Nav.Link>]
                        </p>
                        <p lang={x.item.data.item.locale} style={{marginLeft: '1em'}}>{x.item.data.item.description}</p>
                    </div>
            )}
            <p><a href="#" onClick={e => moreItems(e)} style={{visibility: itemsReachedEnd ? 'hidden' : 'visible'}}>More...</a>{" "}
                <a href={`#/create/for-folder/${serializeItemRef(id)}`}>Create</a></p></>}
            </TabPanel>
            <TabPanel>
                <h3>Comments</h3>
                {comments === undefined ? <p>Loading...</p> : comments.map(x => 
                    <div key={serializeItemRef(x.id)}>
                        {/* FIXME: `!` in `props.defaultAgent!` */}
                        <p lang={x.item.data.item.locale}>
                            <UpDown
                                parent={{id}}
                                item={x}
                                agent={props.defaultAgent!}
                                userVote={userVoteComments[serializeItemRef(x.id)]}
                                totalVotes={totalVotesComments[serializeItemRef(x.id)]}
                                onSetUserVote={(id: ItemRef, v: number) =>
                                    setUserVoteComments({...userVoteComments, [serializeItemRef(id)]: v})}
                                onSetTotalVotes={(id: ItemRef, v: {up: number, down: number}) =>
                                    setTotalVotesComments({...totalVotesComments, [serializeItemRef(id)]: v})}
                                onUpdateList={() => xdata.comments().then(x => setComments(x))}
                                isComment={true}
                            />
                            {x.item.data.item.price ? <>({x.item.data.item.price} ICP) </> : ""}
                            {(x.item.data.item.details as any).link ? <a href={(x.item.data.item.details as any).link}>{x.item.data.item.title}</a> : x.item.data.item.title}
                            {" "}<a href={`#/item/${serializeItemRef(x.id)}`} title="Homepage">[H]</a>
                        </p>
                        <p lang={x.item.data.item.locale} style={{marginLeft: '1em'}}>{x.item.data.item.description}</p>
                    </div>
                )}
                <p><a href="#" onClick={e => moreComments(e)} style={{visibility: commentsReachedEnd ? 'hidden' : 'visible'}}>More...</a>{" "}
                    <a href={`#/create/comment/${serializeItemRef(id)}`}>Create</a></p>
                <h3>Comment on</h3>
                <p><small>Voting in this stream not yet implemented.</small></p>
                {antiComments === undefined ? <p>Loading...</p> : antiComments.map((item: {order: string, id: ItemRef, item: ItemTransfer}) => 
                    <div key={serializeItemRef(item.id)}>
                        <p lang={item.item.data.item.locale}>
                            {item.item.data.item.price ? <>({item.item.data.item.price} ICP) </> : ""}
                            {(item.item.data.item.details as any).link ? <a href={(item.item.data.item.details as any).link}>{item.item.data.item.title}</a> : item.item.data.item.title}
                            {" "}<a href={`#/item/${serializeItemRef(item.id)}`} title="Homepage">[H]</a>
                        </p>
                        <p lang={item.item.data.item.locale} style={{marginLeft: '1em'}}>{item.item.data.item.description}</p>
                    </div>
                )}
                <p><a href="#" onClick={e => moreAntiComments(e)} style={{visibility: antiCommentsReachedEnd ? 'hidden' : 'visible'}}>More...</a>{" "}</p>
            </TabPanel>
        </Tabs>
    </>
}