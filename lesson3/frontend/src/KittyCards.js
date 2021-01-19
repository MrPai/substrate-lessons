import React, {useEffect, useState} from 'react';
import { Button, Card, Grid, Message, Modal, Form, Label } from 'semantic-ui-react';

import KittyAvatar from './KittyAvatar';
import { TxButton } from './substrate-lib/components';
import {useSubstrate} from "./substrate-lib";

// --- About Modal ---

const TransferModal = props => {
  const { kitty, accountPair, setStatus } = props;
  const [open, setOpen] = React.useState(false);
  const [formValue, setFormValue] = React.useState({});


  const formChange = key => (ev, el) => {
    /* TODO: 加代码 */
      setFormValue(el.value);

  };

  const confirmAndClose = (unsub) => {
    unsub();
    setOpen(false);
  };

  return <Modal onClose={() => setOpen(false)} onOpen={() => setOpen(true)} open={open}
    trigger={<Button basic color='blue'>转让</Button>}>
    <Modal.Header>毛孩转让</Modal.Header>
    <Modal.Content><Form>
      <Form.Input fluid label='毛孩 ID' readOnly value={kitty}/>
      <Form.Input fluid label='转让对象' placeholder='对方地址' onChange={formChange('target')}/>
    </Form></Modal.Content>
    <Modal.Actions>
      <Button basic color='grey' onClick={() => setOpen(false)}>取消</Button>
      <TxButton
        accountPair={accountPair} label='确认转让' type='SIGNED-TX' setStatus={setStatus}
        onClick={confirmAndClose}
        attrs={{
          palletRpc: 'kittiesModule',
          callable: 'transfer',
          inputParams: [formValue, kitty],
          paramFields: [true, true]
        }}
      />
    </Modal.Actions>
  </Modal>;
};

// --- About Kitty Card ---

const KittyCard = props => {
  /*
    TODO: 加代码。这里会 UI 显示一张 `KittyCard` 是怎么样的。这里会用到：
    ```
    <KittyAvatar dna={dna} /> - 来描绘一只猫咪
    <TransferModal kitty={kitty} accountPair={accountPair} setStatus={setStatus}/> - 来作转让的弹出层
    ```
  */
    const { dna,kitty,accountPair,setStatus } = props;
    let mine = `我的`;
  return <Grid.Column width={5}>
      <KittyAvatar dna={dna} />
      <TransferModal kitty={kitty} accountPair={accountPair} setStatus={setStatus}/>
      <div>KittyIndex: {kitty}</div>
      <div>{mine}</div>
  </Grid.Column>;
};

const FILTERED_EVENTS = [
    'system:ExtrinsicSuccess:: (phase={"ApplyExtrinsic":0})',
    'system:ExtrinsicSuccess:: (phase={"ApplyExtrinsic":1})'
];
let current_index = 0;
//储存每个账户所包含的kitty list
let account_map = new Map();
//储存每个kitty_index对应的kitty信息
let kitty_map = new Map();
//TODO Pai：由于后端没有有效的方法仅仅依据account获取所有kittyIndex，因此，这里的事件只能捕获新生成的kitty，暂时不实现获取历史中已有的kitty
//存储每个用户拥有的kitty_images
let kitty_images_map = new Map();

const KittyCards = props => {
  const { kitties, accountPair, setStatus } = props;
  /* TODO: 加代码。这里会枚举所有的 `KittyCard` */
    const { api } = useSubstrate();

    useEffect(() => {
        let unsub = null;
        const allEvents = async () => {
            unsub = await api.query.system.events(events => {
                // loop through the Vec<EventRecord>
                events.forEach(record => {
                    // extract the phase, event and the event types
                    const { event, phase } = record;
                    const types = event.typeDef;
                    // show what we are busy with
                    const eventName = `${event.section}:${
                        event.method
                        }:: (phase=${phase.toString()})`;

                    if (FILTERED_EVENTS.includes(eventName)) return;
                    const [_,kittyindex] = event.data;
                    current_index = kittyindex.toString();
                });
            });
        };

        allEvents();
        return () => unsub && unsub();
    }, [api.query.system]);
    current_index = parseInt(current_index);
    let kitty_images = [];
    if (accountPair!=null) {
        let indexes = account_map.get(accountPair.address);
        if (indexes == null) indexes = [];
        if (current_index > 0 && indexes.indexOf(current_index) < 0){
            indexes.push(current_index);
            account_map.set(accountPair.address,indexes);
            current_index = 0;
        }
        // console.log(indexes);
        kitty_images = kitty_images_map.get(accountPair.address);
        if (kitty_images == null) kitty_images = [];
        if(indexes.length > 0 && indexes.length > kitty_images.length) {
            kitty_images = [];
            api.query.kittiesModule.kitties.multi(indexes, (data) => {
                for (let i = 0; i < data.length; i++) {
                    //根据kittyIndex逐个获取kitty的dna信息
                    let kitty_index = indexes[i];
                    let dna = data[i].__private_45_raw;
                    kitty_images.push(<KittyCard dna={dna} kitty={kitty_index} accountPair={accountPair} setStatus={setStatus}/>);
                }
            });
            kitty_images_map.set(accountPair.address,kitty_images);
        }
    }
  return kitty_images;
};


export default KittyCards;
