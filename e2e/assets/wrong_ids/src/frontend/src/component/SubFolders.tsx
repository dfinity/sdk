import * as React from "react";
import { useEffect, useState } from "react";
import { AppData } from "../DataDispatcher";
import { useNavigate, useParams } from "react-router-dom";
import { ItemRef, serializeItemRef } from "../data/Data";
import { ItemData, ItemTransfer } from "../../../declarations/CanDBPartition/CanDBPartition.did";
import ItemType from "./misc/ItemType";
import { Agent } from "@dfinity/agent";

export default function SubFolders(props: {defaultAgent: Agent | undefined, 'data-dir': 'sub' | 'super'}) { // TODO: any
    const { id } = useParams();
    const [xdata, setXData] = useState<any>(undefined);
    const [title, setTitle] = useState("");
    const [folders, setFolders] = useState<{order: string, id: ItemRef, item: ItemTransfer}[] | undefined>([]);
    const [itemsLast, setItemsLast] = useState("");
    const [itemsReachedEnd, setItemsReachedEnd] = useState(false);
    const [streamKind, setStreamKind] = useState<"t" | "v">("v"); // time, votes
    function updateStreamKind(e) {
        setStreamKind(e.currentTarget.value);
    }

    const navigate = useNavigate();
    useEffect(() => {
        if (id !== undefined) {
            AppData.create(props.defaultAgent!, id, streamKind).then(data => { // TODO: `!`
                data.title().then(x => setTitle(x));
                if (props['data-dir'] == 'super') {
                    data.superFolders().then(x => {
                        setFolders(x); // TODO: SUPER-folders
                        // TODO: duplicate code
                        if (x.length !== 0) {
                            setItemsLast(x[x.length - 1].order);
                        }
                    });
                } else {
                    data.subFolders().then(x => {
                        setFolders(x);
                        // TODO: duplicate code
                        if (x.length !== 0) {
                            setItemsLast(x[x.length - 1].order);
                        }
                    });
                }
                setXData(data);
            });
        }
    }, [id, props.defaultAgent, streamKind]);

    function moreItems(event: any) {
        event.preventDefault();
        if (folders?.length === 0) {
            return;
        }
        const lowerBound = itemsLast + 'x';
        console.log('lowerBound', lowerBound)
        const promise = props['data-dir'] == 'super'
            ? xdata.superFolders({lowerBound, limit: 10}) : xdata.subFolders({lowerBound, limit: 10});
        promise.then(x => {
            console.log('X', x)
            setFolders(folders?.concat(x)); // TODO: `?`?
            if (x.length !== 0) {
                setItemsLast(x[x.length - 1].order); // duplicate code
            } else {
                setItemsReachedEnd(true);
            }
        });
    }

    return (
        <>
            <h2>{props['data-dir'] == 'super' ? "Super-folders" : "Subfolders"} of: <a href='#' onClick={() => navigate(`/item/`+id)}>{title}</a></h2>
            <p>Sort by:{" "}
                <label><input type="radio" name="stream" value="t" onChange={updateStreamKind} checked={streamKind == "t"}/> time</label>{" "}
                <label><input type="radio" name="stream" value="v" onChange={updateStreamKind} checked={streamKind == "v"}/> votes</label>{" "}
            </p>
           <ul>
                {folders !== undefined && folders.map(x =>
                    <li key={serializeItemRef(x.id as any)}>
                        <p>
                            <ItemType item={x.item}/>
                            <a lang={x.item.data.item.locale} href={`#/item/${serializeItemRef(x.id as any)}`}>{x.item.data.item.title}</a>
                        </p>
                        {x.item.data.item.description ? <p lang={x.item.data.item.locale}><small>{x.item.data.item.description}</small></p> : ""}
                    </li>)}
            </ul>
            <p><a href="#" onClick={e => moreItems(e)} style={{visibility: itemsReachedEnd ? 'hidden' : 'visible'}}>More...</a>{" "}
                <a href={`#/create/for-folder/${id}`}>Create</a></p>
        </>
    );
}
