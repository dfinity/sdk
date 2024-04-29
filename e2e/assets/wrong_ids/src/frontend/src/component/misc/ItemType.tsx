import * as React from "react";
import { ItemTransfer } from "../../../../declarations/CanDBPartition/CanDBPartition.did";

export default function ItemType(props: {item: ItemTransfer}) { // TODO: Is it the right type of argument?
    // FIXME
    return <>
        {props.item && (props.item.communal ?
            <span title="Communal folder">&#x1f465;</span> :
            <span title="Owned folder">&#x1f464;</span>)}
    </>
    // return <>
    // {props.item && (
    //     <span title="Owned item">&#x1f464;</span>)}
    // </>
}